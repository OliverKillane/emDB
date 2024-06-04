use std::collections::HashMap;

use super::SingleOp;
use crate::{
    columns::ColKind,
    groups::{Field, FieldIndex, FieldName, Group, Groups, MutImmut},
    namer::CodeNamer,
};
use proc_macro2::TokenStream;
use quote::quote;
use quote_debug::Tokens;
use syn::Type;

fn generate_get_fields<'a>(
    groups: &'a Groups,
    namer: &'a CodeNamer,
) -> impl Iterator<Item = TokenStream> + 'a {
    groups.idents.iter().map(|(field_name, field_index)| {
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

/// Used to generate the field types for get operations on a table
pub fn get_struct_fields<'a>(
    groups: &'a Groups,
    namer: &'a CodeNamer,
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

pub fn generate_get_struct_fields<'a>(
    groups: &'a Groups,
    namer: &'a CodeNamer,
) -> Vec<TokenStream> {
    fn append<Col: ColKind>(
        fs: &mut Vec<TokenStream>,
        col: &Col,
        fields: &MutImmut<Vec<Field>>,
        namer: &CodeNamer,
    ) {
        for Field { name, ty } in &fields.mut_fields {
            fs.push(quote!(pub #name: #ty));
        }
        for field @ Field { name, .. } in &fields.imm_fields {
            let ty_trans = col.convert_imm_type(field, namer);
            fs.push(quote!(pub #name: #ty_trans));
        }
    }
    let mut def_fields = Vec::with_capacity(groups.idents.len());
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

pub fn generate(groups: &Groups, namer: &CodeNamer) -> SingleOp {
    let CodeNamer {
        type_key_error,
        type_key,
        struct_window,
        pulpit_path,
        name_primary_column,
        struct_table_member_columns: table_member_columns,
        mod_columns,
        mod_columns_fn_imm_unpack,
        mod_get,
        mod_get_struct_get,
        lifetime_imm,
        struct_window_method_get: method_get,
        name_phantom_member,
        ..
    } = namer;

    let include_lifetime = groups.primary.col.requires_get_lifetime()
        || groups
            .assoc
            .iter()
            .any(|Group { col, fields: _ }| col.requires_get_lifetime()); // TODO: implement
    let lifetime = if include_lifetime {
        quote!(<#lifetime_imm>)
    } else {
        quote!()
    };

    let get_struct_fields = generate_get_struct_fields(groups, namer);

    let phantom_get = if include_lifetime && get_struct_fields.is_empty() {
        quote!(pub #name_phantom_member: std::marker::PhantomData<&#lifetime_imm ()>)
    } else {
        quote!()
    };

    let assoc_cols = (0..groups.assoc.len()).map(|ind| {
        let name = namer.name_assoc_column(ind);
        quote!(let #name = unsafe { self.#table_member_columns.#name.assoc_get(index) }.convert_imm(#mod_columns::#name::#mod_columns_fn_imm_unpack))
    });
    let get_fields_stream = generate_get_fields(groups, namer).collect::<Vec<_>>();
    let get_fields = if get_fields_stream.is_empty() {
        quote!(#name_phantom_member: std::marker::PhantomData)
    } else {
        quote!(#(#get_fields_stream,)*)
    };

    SingleOp {
        op_mod: quote! {
            pub mod #mod_get {
                pub struct #mod_get_struct_get #lifetime {
                    #(#get_struct_fields,)*
                    #phantom_get
                }
            }
        }
        .into(),
        op_impl: quote! {
            impl <#lifetime_imm> #struct_window<#lifetime_imm> {
                pub fn #method_get(&self, key: #type_key) -> Result<#mod_get::#mod_get_struct_get #lifetime, #type_key_error> {
                    let #pulpit_path::column::Entry {index, data: #name_primary_column} = match self.#table_member_columns.#name_primary_column.get(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(#type_key_error),
                    };
                    let #name_primary_column = #name_primary_column.convert_imm(#mod_columns::#name_primary_column::#mod_columns_fn_imm_unpack);
                    #(#assoc_cols;)*

                    Ok(#mod_get::#mod_get_struct_get {
                        #get_fields
                    })
                }
            }
        }
        .into(),
    }
}
