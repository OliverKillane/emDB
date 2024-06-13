use std::collections::HashMap;

use super::SingleOp;
use crate::{
    columns::ColKind,
    groups::{Field, FieldIndex, FieldIndexInner, FieldName, Group, Groups, MutImmut},
    namer::CodeNamer,
};
use proc_macro2::TokenStream;
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ImplItemFn, ItemStruct, Type};

fn access<Col: ColKind>(
    group: &Group<Col>,
    field_index: &FieldIndexInner,
    namer: &CodeNamer,
) -> Tokens<Type> {
    let field = group.get_field(field_index);
    if field_index.imm {
        group.col.convert_imm_type(field, namer)
    } else {
        field.ty.clone()
    }
}

pub struct Get {
    pub alias: Ident,
    pub fields: Vec<FieldName>,
}

struct GetGen {
    struct_def: Tokens<ItemStruct>,
    impl_def: Tokens<ImplItemFn>,
}

/// Provides a map of types for the entire table - used in the `TableGet` scalar type in emDB 
pub fn get_struct_fields(
    groups: &Groups,
    namer: &CodeNamer,
) -> HashMap<FieldName, Tokens<Type>> {
    fn append<Col: ColKind>(
        fs: &mut HashMap<FieldName, Tokens<Type>>,
        col: &Col,
        fields: &MutImmut<Vec<Field>>,
        namer: &CodeNamer,
    ) {
        for Field { name, ty } in &fields.mut_fields {
            fs.insert(name.clone(), ty.clone());
        }
        for field @ Field { name, .. } in &fields.imm_fields {
            fs.insert(name.clone(), col.convert_imm_type(field, namer));
        }
    }
    let mut def_fields = HashMap::with_capacity(groups.idents.len());
    append(
        &mut def_fields,
        &groups.primary.col,
        &groups.primary.fields,
        namer,
    );

    for Group { col, fields } in &groups.assoc {
        append(&mut def_fields, col, fields, namer);
    }
    def_fields
}

impl Get {
    fn generate_get_fields<'a>(
        &'a self,
        groups: &'a Groups,
        namer: &'a CodeNamer,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields.iter().map(|field_name| {
            let field_index = groups.idents.get(field_name).unwrap();
            let data = match field_index {
                FieldIndex::Primary(_) => namer.name_primary_column.clone(),
                FieldIndex::Assoc {
                    assoc_ind,
                    inner: _,
                } => namer.name_assoc_column(*assoc_ind),
            };

            let imm_access = if field_index.is_imm() {
                quote!(imm_data)
            } else {
                quote!(mut_data)
            };

            quote!(#field_name: #data.#imm_access.#field_name)
        })
    }

    pub fn get_struct_fields<'a>(
        &'a self,
        groups: &'a Groups,
        namer: &'a CodeNamer,
    ) -> impl Iterator<Item = (FieldName, Tokens<Type>)> + 'a {
        // NOTE: this is inefficient, requires traversing over all fields twice

        self.fields.iter().map(|field_name| {
            let field_index = groups.idents.get(field_name).unwrap();
            match field_index {
                FieldIndex::Primary(inner_index) => (
                    field_name.clone(),
                    access(&groups.primary, inner_index, namer),
                ),
                FieldIndex::Assoc { assoc_ind, inner } => {
                    let group = groups.assoc.get(*assoc_ind).unwrap();
                    (field_name.clone(), access(group, inner, namer))
                }
            }
        })
    }

    pub fn generate_get_struct_fields(
        &self,
        groups: &Groups,
        namer: &CodeNamer,
    ) -> Vec<TokenStream> {
        self.get_struct_fields(groups, namer)
            .map(|(name, ty)| quote!(pub #name: #ty))
            .collect()
    }

    fn generate(
        &self,
        include_lifetime: bool,
        groups: &Groups,
        namer: &CodeNamer,
        op_attrs: &TokenStream,
    ) -> GetGen {
        let CodeNamer {
            type_key_error,
            type_key,
            pulpit_path,
            name_primary_column,
            struct_table_member_columns: table_member_columns,
            mod_columns,
            mod_columns_fn_imm_unpack,
            mod_get,
            lifetime_imm,
            name_phantom_member,
            ..
        } = namer;
        let get_struct_name = &self.alias;

        let (lifetime, phantom_def, phantom_get) = if include_lifetime {
            (
                quote!(<#lifetime_imm>),
                quote!(pub #name_phantom_member: std::marker::PhantomData<&#lifetime_imm ()>),
                quote!(#name_phantom_member: std::marker::PhantomData),
            )
        } else {
            (quote!(), quote!(), quote!())
        };

        let get_struct_fields = self.generate_get_struct_fields(groups, namer);
        let assoc_cols = (0..groups.assoc.len()).map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(let #name = unsafe { self.#table_member_columns.#name.assoc_get(index) }.convert_imm(#mod_columns::#name::#mod_columns_fn_imm_unpack))
        });
        let get_fields_stream = self.generate_get_fields(groups, namer);
        let get_method_name = self.alias.clone();

        GetGen {
            struct_def: quote!{
                pub struct #get_struct_name #lifetime {
                    #(#get_struct_fields,)*
                    #phantom_def
                }
            }.into(),
            impl_def: quote!{
                #op_attrs
                pub fn #get_method_name(&self, key: #type_key) -> Result<#mod_get::#get_struct_name #lifetime, #type_key_error> {
                    let #pulpit_path::column::Entry {index, data: #name_primary_column} = match self.#table_member_columns.#name_primary_column.get(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(#type_key_error),
                    };
                    let #name_primary_column = #name_primary_column.convert_imm(#mod_columns::#name_primary_column::#mod_columns_fn_imm_unpack);
                    #(#assoc_cols;)*

                    Ok(#mod_get::#get_struct_name {
                        #(#get_fields_stream,)*
                        #phantom_get
                    })
                }
            }.into(),
        }
    }
}

pub fn generate(groups: &Groups, namer: &CodeNamer, get_ops: &[Get],op_attrs: &TokenStream) -> SingleOp {
    let CodeNamer {
        struct_window,
        mod_get,
        lifetime_imm,
        ..
    } = namer;

    let include_lifetime = groups.primary.col.requires_get_lifetime()
        || groups
            .assoc
            .iter()
            .any(|Group { col, fields: _ }| col.requires_get_lifetime()); // TODO: implement

    let (structs, impl_fns): (Vec<_>,Vec<_>) = get_ops.iter().map(|op| op.generate(include_lifetime, groups, namer, op_attrs)).map(|GetGen { struct_def, impl_def }| (struct_def, impl_def)).unzip();

    SingleOp {
        op_mod: quote!{
            pub mod #mod_get {
                #(#structs)*
            }
        }.into(),
        op_impl: quote! {
            impl <#lifetime_imm> #struct_window<#lifetime_imm> {
                #(#impl_fns)*
            }
        }.into()
    }
}
