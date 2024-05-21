use std::{collections::HashMap, iter::once};

use proc_macro2::TokenStream;
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemType, Type};

use super::namer::CodeNamer;

pub type FieldName = Ident;

pub struct MutImmut<Data> {
    pub imm_fields: Data,
    pub mut_fields: Data,
}

pub struct Field {
    pub name: FieldName,
    pub ty: Tokens<Type>,
}

pub struct Group<Col: ColKind> {
    pub col: Col,
    pub fields: MutImmut<Vec<Field>>,
}

impl<Col: ColKind> Group<Col> {
    fn get_type<'a>(&'a self, index: &FieldIndexInner) -> Option<&'a Tokens<Type>> {
        if index.imm {
            &self.fields.imm_fields
        } else {
            &self.fields.mut_fields
        }
        .get(index.field_num)
        .map(|f| &f.ty)
    }
}

pub struct Groups<Primary: PrimaryKind> {
    pub idents: HashMap<Ident, FieldIndex>,
    pub primary: Group<Primary>,
    pub assoc: Vec<Group<Primary::Assoc>>,
}

impl<Primary: PrimaryKind> Groups<Primary> {
    // get type from field

    pub fn get_field_index(&self, field: &FieldName) -> Option<&FieldIndex> {
        self.idents.get(field)
    }

    pub fn get_type(&self, index: &FieldIndex) -> Option<&Tokens<Type>> {
        match index {
            FieldIndex::Primary(inner) => self.primary.get_type(inner),
            FieldIndex::Assoc { assoc_ind, inner } => self
                .assoc
                .get(*assoc_ind)
                .map(|f| f.get_type(inner))
                .flatten(),
        }
    }

    pub fn get_typefield(&self, field: &FieldName) -> Option<&Tokens<Type>> {
        self.get_field_index(field)
            .map(|f| self.get_type(f))
            .flatten()
    }
}

pub struct FieldIndexInner {
    pub imm: bool,
    pub field_num: usize,
}

pub enum FieldIndex {
    Primary(FieldIndexInner),
    Assoc {
        assoc_ind: usize,
        inner: FieldIndexInner,
    },
}

impl FieldIndex {
    pub fn is_imm(&self) -> bool {
        match self {
            FieldIndex::Primary(inner) => inner.imm,
            FieldIndex::Assoc { inner, .. } => inner.imm,
        }
    }
}

pub struct GroupsDef {
    pub columns_struct: Tokens<ItemStruct>,
    pub columns_impl: Tokens<ItemImpl>,
    pub window_struct: Tokens<ItemStruct>,
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

pub trait AssocKind: ColKind {}

pub trait PrimaryKind: ColKind {
    const TRANSACTIONS: bool;
    const DELETIONS: bool;
    type Assoc: AssocKind;
}

impl<Col: ColKind> Group<Col> {
    fn column_type(&self, group_name: Ident, namer: &CodeNamer) -> Tokens<ItemMod> {
        let imm_struct_name = namer.mod_columns_struct_imm();
        let mut_struct_name = namer.mod_columns_struct_mut();

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
    pub fn column_types(&self, namer: &CodeNamer) -> Tokens<ItemMod> {
        let mod_name = namer.mod_columns();

        let primary_mod = self.primary.column_type(namer.name_primary_column(), namer);
        let assoc_mods = self
            .assoc
            .iter()
            .enumerate()
            .map(|(ind, grp)| grp.column_type(namer.name_assoc_column(ind), namer));
        quote! {
            mod #mod_name {
                //! Column types to be used for storage in each column.
                #primary_mod
                #(#assoc_mods)*
            }
        }
        .into()
    }

    pub fn key_type(&self, namer: &CodeNamer) -> Tokens<ItemType> {
        let col_types = namer.mod_columns();
        let primary_mod = namer.name_primary_column();
        let imm_struct_name = namer.mod_columns_struct_imm();
        let mut_struct_name = namer.mod_columns_struct_mut();
        let primary_type = self.primary.col.generate_column_type(
            namer,
            quote!(#col_types::#primary_mod::#imm_struct_name).into(),
            quote!(#col_types::#primary_mod::#mut_struct_name).into(),
        );
        let pulpit_path = namer.pulpit_path();
        let key_type = namer.type_key();
        quote! {
            /// The key for accessing rows (delete, update, get)
            pub type #key_type = <#primary_type as #pulpit_path::column::Keyable>::Key;
        }
        .into()
    }

    pub fn columns_definition(&self, namer: &CodeNamer) -> GroupsDef {
        let col_types = namer.mod_columns();
        let primary_mod = namer.name_primary_column();

        let imm_struct_name = namer.mod_columns_struct_imm();
        let mut_struct_name = namer.mod_columns_struct_mut();

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
                let assoc_name = namer.name_assoc_column(ind);
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

        let column_holder = namer.struct_column_holder();
        let window_holder = namer.struct_window_holder();

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
