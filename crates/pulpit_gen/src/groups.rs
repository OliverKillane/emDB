use std::{collections::HashMap, iter::once};

use proc_macro2::TokenStream;
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ItemImpl, ItemMod, ItemStruct, ItemType, Type};

use crate::{
    columns::{Associated, ColKind, ImmConversion, Primary},
    namer::CodeNamer,
};

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

    fn get_members(
        &self,
        placement: impl Fn(FieldIndexInner) -> FieldIndex,
        mapping: &mut HashMap<FieldName, FieldIndex>,
    ) {
        for (ind, field) in self.fields.imm_fields.iter().enumerate() {
            mapping.insert(
                field.name.clone(),
                placement(FieldIndexInner {
                    imm: true,
                    field_num: ind,
                }),
            );
        }
        for (ind, field) in self.fields.mut_fields.iter().enumerate() {
            mapping.insert(
                field.name.clone(),
                placement(FieldIndexInner {
                    imm: false,
                    field_num: ind,
                }),
            );
        }
    }

    pub fn get_field(&self, index: &FieldIndexInner) -> &Field {
        if index.imm {
            &self.fields.imm_fields
        } else {
            &self.fields.mut_fields
        }
        .get(index.field_num)
        .unwrap()
    }
}

pub struct GroupConfig {
    pub primary: Group<Primary>,
    pub assoc: Vec<Group<Associated>>,
}

impl From<GroupConfig> for Groups {
    fn from(GroupConfig { primary, assoc }: GroupConfig) -> Self {
        let mut idents = HashMap::new();
        primary.get_members(FieldIndex::Primary, &mut idents);
        for (ind, group) in assoc.iter().enumerate() {
            group.get_members(
                |index| FieldIndex::Assoc {
                    assoc_ind: ind,
                    inner: index,
                },
                &mut idents,
            );
        }

        Groups {
            idents,
            primary,
            assoc,
        }
    }
}

pub struct Groups {
    pub idents: HashMap<Ident, FieldIndex>,
    pub primary: Group<Primary>,
    pub assoc: Vec<Group<Associated>>,
}

impl Groups {
    // get type from field

    pub fn get_field_index(&self, field: &FieldName) -> Option<&FieldIndex> {
        self.idents.get(field)
    }

    pub fn get_type(&self, index: &FieldIndex) -> Option<&Tokens<Type>> {
        match index {
            FieldIndex::Primary(inner) => self.primary.get_type(inner),
            FieldIndex::Assoc { assoc_ind, inner } => {
                self.assoc.get(*assoc_ind).and_then(|f| f.get_type(inner))
            }
        }
    }

    pub fn get_typefield(&self, field: &FieldName) -> Option<&Tokens<Type>> {
        self.get_field_index(field).and_then(|f| self.get_type(f))
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
    pub window_holder_struct: Tokens<ItemStruct>,
}

impl<Col: ColKind> Group<Col> {
    fn column_type(&self, group_name: Ident, namer: &CodeNamer) -> Tokens<ItemMod> {
        let CodeNamer {
            mod_columns_struct_imm,
            mod_columns_struct_mut,
            ..
        } = namer;

        let MutImmut {
            imm_fields: imm_derives,
            mut_fields: mut_derives,
        } = self.col.derives();

        fn get_tks(Field { name, ty }: &Field) -> TokenStream {
            quote!(pub #name: #ty)
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
                pub struct #mod_columns_struct_imm {
                    #(#imm_fields),*
                }

                #[derive(#(#mut_derives),*)]
                pub struct #mod_columns_struct_mut {
                    #(#mut_fields),*
                }

                #imm_unpacked

                #[inline(always)]
                #unpacker
            }
        }
        .into()
    }
}

impl Groups {
    pub fn column_types(&self, namer: &CodeNamer) -> Tokens<ItemMod> {
        let mod_columns = &namer.mod_columns;

        let primary_mod = self
            .primary
            .column_type(namer.name_primary_column.clone(), namer);
        let assoc_mods = self
            .assoc
            .iter()
            .enumerate()
            .map(|(ind, grp)| grp.column_type(namer.name_assoc_column(ind), namer));
        quote! {
            mod #mod_columns {
                //! Column types to be used for storage in each column.
                #primary_mod
                #(#assoc_mods)*
            }
        }
        .into()
    }

    pub fn key_type(&self, namer: &CodeNamer) -> Tokens<ItemType> {
        let CodeNamer {
            mod_columns,
            name_primary_column,
            mod_columns_struct_imm,
            mod_columns_struct_mut,
            pulpit_path,
            type_key,
            ..
        } = namer;

        let primary_type = self.primary.col.generate_column_type(
            namer,
            quote!(#mod_columns::#name_primary_column::#mod_columns_struct_imm).into(),
            quote!(#mod_columns::#name_primary_column::#mod_columns_struct_mut).into(),
        );
        quote! {
            /// The key for accessing rows (delete, update, get)
            pub type #type_key = <#primary_type as #pulpit_path::column::Keyable>::Key;
        }
        .into()
    }

    pub fn columns_definition(&self, namer: &CodeNamer) -> GroupsDef {
        let CodeNamer {
            mod_columns,
            name_primary_column,
            mod_columns_struct_imm,
            mod_columns_struct_mut,
            pulpit_path,
            struct_column_holder,
            struct_window_holder,
            ..
        } = namer;

        let num_members = self.assoc.len() + 1;
        let mut col_defs = Vec::with_capacity(num_members);
        let mut window_defs = Vec::with_capacity(num_members);
        let mut converts = Vec::with_capacity(num_members);
        let mut news = Vec::with_capacity(num_members);

        for (ty, ty_no_gen, member) in self
            .assoc
            .iter()
            .enumerate()
            .map(|(ind, Group { col, fields: _ })| {
                let assoc_name = namer.name_assoc_column(ind);
                (
                    col.generate_column_type(
                        namer,
                        quote!(#mod_columns::#assoc_name::#mod_columns_struct_imm).into(),
                        quote!(#mod_columns::#assoc_name::#mod_columns_struct_mut).into(),
                    ),
                    col.generate_base_type(namer),
                    assoc_name,
                )
            })
            .chain(once((
                self.primary.col.generate_column_type(
                    namer,
                    quote!(#mod_columns::#name_primary_column::#mod_columns_struct_imm).into(),
                    quote!(#mod_columns::#name_primary_column::#mod_columns_struct_mut).into(),
                ),
                self.primary.col.generate_base_type(namer),
                name_primary_column.clone(),
            )))
        {
            col_defs.push(quote!(#member: #ty));
            window_defs
                .push(quote!(#member: <#ty as #pulpit_path::column::Column>::WindowKind<'imm>));
            converts.push(quote!(#member: self.#member.window()));
            news.push(quote!(#member: #ty_no_gen::new(size_hint)));
        }

        GroupsDef {
            columns_struct: quote! {
                struct #struct_column_holder {
                    #(#col_defs),*
                }
            }
            .into(),
            columns_impl: quote! {
                impl #struct_column_holder {
                    fn new(size_hint: usize) -> Self {
                        Self {
                            #(#news),*
                        }
                    }

                    fn window(&mut self) -> #struct_window_holder<'_> {
                        #struct_window_holder {
                            #(#converts),*
                        }
                    }
                }
            }
            .into(),
            window_holder_struct: quote! {
                struct #struct_window_holder<'imm> {
                    #(#window_defs),*
                }
            }
            .into(),
        }
    }
}
