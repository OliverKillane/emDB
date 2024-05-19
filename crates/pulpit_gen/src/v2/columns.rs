use std::{collections::HashMap, iter::once};

use bimap::BiMap;
use proc_macro2::TokenStream;
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemType, Path, Type};

use super::namer::CodeNamer;

pub type FieldName = Ident;

pub struct MutImmut<Data> {
    imm_fields: Data,
    mut_fields: Data,
}

pub struct Field {
    name: FieldName,
    ty: Tokens<Type>,
}

pub struct Group<Col: ColKind> {
    col: Col,
    fields: MutImmut<Vec<Field>>,
}

pub struct Groups<Primary: PrimaryKind> {
    pub idents: BiMap<Ident, FieldIndex>,
    pub primary: Group<Primary>,
    pub assoc: Vec<Group<Primary::Assoc>>,
}

pub struct FieldIndexInner {
    imm: bool,
    field_num: usize,
}

pub enum FieldIndex {
    Primary(FieldIndexInner),
    Assoc {
        assoc_ind: usize,
        inner: FieldIndexInner,
    },
}

struct GroupsDef {
    columns_struct: Tokens<ItemStruct>,
    columns_impl: Tokens<ItemImpl>,
    window_struct: Tokens<ItemStruct>,
}

struct ImmConversion {
    imm_unpacked: Tokens<ItemStruct>,
    unpacker: Tokens<ItemFn>,
}

trait ColKind {
    fn derives(&self) -> MutImmut<Vec<Ident>>;
    fn generate_column_type(
        &self,
        namer: &CodeNamer,
        imm_type: Tokens<Type>,
        mut_type: Tokens<Type>,
    ) -> Tokens<Type>;
    fn convert_imm(&self, namer: &CodeNamer, imm_fields: &[Field]) -> ImmConversion;
}

pub trait AssocKind: ColKind {
    const DELETIONS: bool;
}

pub trait PrimaryKind: ColKind {
    const TRANSACTIONS: bool;
    const DELETIONS: bool;
    type Assoc: AssocKind<DELETIONS=Self::DELETIONS>;
}

impl<Col: ColKind> Group<Col> {
    fn column_type(&self, group_name: Ident, namer: &CodeNamer) -> Tokens<ItemMod> {
        let imm_struct_name = namer.column_types_imm_struct();
        let mut_struct_name = namer.column_types_mut_struct();

        let MutImmut {
            imm_fields: imm_derives,
            mut_fields: mut_derives,
        } = self.col.derives();

        fn get_tks(Field { name, ty }: &Field) -> TokenStream {
            quote!(#name: #ty)
        }

        let MutImmut {
            imm_fields,
            mut_fields,
        } = &self.fields;
        let imm_fields = imm_fields.iter().map(get_tks);
        let mut_fields = mut_fields.iter().map(get_tks);

        let ImmConversion {
            imm_unpacked,
            unpacker,
        } = self.col.convert_imm(namer, &self.fields.imm_fields);

        quote! {
            pub mod #group_name {
                #[derive(#(#imm_derives),*)]
                pub struct #imm_struct_name {
                    #(#imm_fields),*
                }

                #[derive(#(#mut_derives),*)]
                pub struct #mut_struct_name {
                    #(#mut_fields),*
                }

                #imm_unpacked

                #unpacker
            }
        }
        .into()
    }
}

impl<Primary: PrimaryKind> Groups<Primary> {
    fn column_types(&self, namer: &CodeNamer) -> Tokens<ItemMod> {
        let mod_name = namer.column_types_mod();

        let primary_mod = self
            .primary
            .column_type(namer.column_types_primary_mod(), namer);
        let assoc_mods = self
            .assoc
            .iter()
            .enumerate()
            .map(|(ind, grp)| grp.column_type(namer.column_types_assoc_mod(ind), namer));
        quote! {
            mod #mod_name {
                //! Column types to be used for storage in each column.

                #primary_mod
                #(#assoc_mods)*
            }
        }
        .into()
    }

    fn key_type(&self, namer: &CodeNamer) -> Tokens<ItemType> {
        let col_types = namer.column_types_mod();
        let primary_mod = namer.column_types_primary_mod();
        let imm_struct_name = namer.column_types_imm_struct();
        let mut_struct_name = namer.column_types_mut_struct();
        let primary_type = self.primary.col.generate_column_type(
            namer,
            quote!(#col_types::#primary_mod::#imm_struct_name).into(),
            quote!(#col_types::#primary_mod::#mut_struct_name).into(),
        );
        let pulpit_path = namer.pulpit_path();

        quote! {
            /// The key for accessing rows (delete, update, get)
            pub type Key = <#primary_type as #pulpit_path::column::Keyable>::Key;
        }
        .into()
    }

    fn columns_definition(&self, namer: &CodeNamer) -> GroupsDef {
        let col_types = namer.column_types_mod();
        let primary_mod = namer.column_types_primary_mod();

        let imm_struct_name = namer.column_types_imm_struct();
        let mut_struct_name = namer.column_types_mut_struct();

        let pulpit_path = namer.pulpit_path();

        let num_members = self.assoc.len() + 1;
        let mut col_defs = Vec::with_capacity(num_members);
        let mut window_defs = Vec::with_capacity(num_members);
        let mut converts = Vec::with_capacity(num_members);
        let mut news = Vec::with_capacity(num_members);

        for (ty, member) in self
            .assoc
            .iter()
            .enumerate()
            .map(|(ind, Group { col, fields })| {
                let assoc_name = namer.column_types_assoc_mod(ind);
                (
                    col.generate_column_type(
                        namer,
                        quote!(#col_types::#assoc_name::#imm_struct_name).into(),
                        quote!(#col_types::#assoc_name::#mut_struct_name).into(),
                    ),
                    assoc_name,
                )
            })
            .chain(once((
                self.primary.col.generate_column_type(
                    namer,
                    quote!(#col_types::#primary_mod::#imm_struct_name).into(),
                    quote!(#col_types::#primary_mod::#mut_struct_name).into(),
                ),
                primary_mod,
            )))
        {
            col_defs.push(quote!(#member: #ty));
            window_defs
                .push(quote!(#member: <#ty as #pulpit_path::column::Column>::WindowKind<'imm>));
            converts.push(quote!(#member: self.#member.window()));
            news.push(quote!(#member: #ty::new(size_hint)));
        }

        let column_holder = namer.column_holder();
        let window_holder = namer.window_holder();

        GroupsDef {
            columns_struct: quote! {
                struct #column_holder {
                    #(#col_defs),*
                }
            }
            .into(),
            columns_impl: quote! {
                impl #column_holder {
                    fn new(size_hint: usize) -> Self {
                        Self {
                            #(#news),*
                        }
                    }

                    fn window(&mut self) -> #window_holder<'_> {
                        #window_holder {
                            #(#converts),*
                        }
                    }
                }
            }
            .into(),
            window_struct: quote! {
                struct #window_holder<'imm> {
                    #(#window_defs),*
                }
            }
            .into(),
        }
    }
}
