use std::collections::HashMap;

use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ItemStruct};

use super::{
    columns::{FieldName, Groups, PrimaryKind},
    namer::CodeNamer,
};

pub struct Unique {
    pub alias: Ident,
}

pub fn generate<Primary: PrimaryKind>(
    uniques: &HashMap<FieldName, Unique>,
    groups: &Groups<Primary>,
    namer: &CodeNamer,
) -> Tokens<ItemStruct> {
    let pulpit_path = namer.pulpit_path();
    let unique_struct = namer.mod_columns();
    let key_type = namer.type_key();
    let unique_fields = uniques.iter().map(|(alias, _)| {
        let ty = groups.get_typefield(&alias).unwrap();
        quote!(#alias: #pulpit_path::access::Unique<#ty, #key_type>)
    });

    quote! {
        pub struct #unique_struct {
            #(#unique_fields),*
        }
    }
    .into()
}
