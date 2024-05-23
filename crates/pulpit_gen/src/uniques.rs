use std::collections::HashMap;

use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ItemImpl, ItemStruct};

use super::{
    columns::PrimaryKind,
    groups::{FieldName, Groups},
    namer::CodeNamer,
};

pub struct Unique {
    pub alias: Ident,
}

pub struct UniqueDec {
    pub unique_struct: Tokens<ItemStruct>,
    pub unique_impl: Tokens<ItemImpl>,
}

pub fn generate<Primary: PrimaryKind>(
    uniques: &HashMap<FieldName, Unique>,
    groups: &Groups<Primary>,
    namer: &CodeNamer,
) -> UniqueDec {
    let CodeNamer {
        pulpit_path,
        struct_unique,
        type_key,
        ..
    } = namer;

    let unique_fields_def = uniques.iter().map(|(alias, _)| {
        let ty = groups.get_typefield(alias).unwrap();
        quote!(#alias: #pulpit_path::access::Unique<#ty, #type_key>)
    });
    let unique_fields_impl = uniques
        .iter()
        .map(|(alias, _)| quote!(#alias: #pulpit_path::access::Unique::new(size_hint)));

    UniqueDec {
        unique_struct: quote! {
            struct #struct_unique {
                #(#unique_fields_def),*
            }
        }
        .into(),
        unique_impl: quote! {
            impl #struct_unique {
                fn new(size_hint: usize) -> Self {
                    Self {
                        #(#unique_fields_impl),*
                    }
                }
            }
        }
        .into(),
    }
}
