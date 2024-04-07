use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use std::collections::LinkedList;
use syn::{parse2, spanned::Spanned, Fields, ItemEnum, Type, Variant};

use crate::macro_comm::extract_syn;

pub fn interface(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, LinkedList<Diagnostic>> {
    parse_attrs(attr)?;
    let enum_def = extract_syn(
        item.clone(),
        &Ident::new("quick_from", Span::call_site()),
        parse2::<ItemEnum>,
    )?;

    let from_defs = impl_from(&enum_def);

    Ok(quote! {
        #item
        #from_defs
    })
}

fn parse_attrs(attr: TokenStream) -> Result<(), LinkedList<Diagnostic>> {
    if attr.is_empty() {
        Ok(())
    } else {
        Err(LinkedList::from([Diagnostic::spanned(
            attr.span(),
            Level::Error,
            "This macro takes no arguments".to_owned(),
        )]))
    }
}

fn get_var_name_type(var: &Variant) -> Option<(Ident, Type)> {
    if let Fields::Unnamed(fs) = &var.fields {
        if fs.unnamed.len() == 1 {
            return Some((var.ident.clone(), fs.unnamed.first().unwrap().ty.clone()));
        }
    }
    None
}

fn impl_from(enum_def: &ItemEnum) -> TokenStream {
    let enum_name = &enum_def.ident;
    let from_impls = enum_def
        .variants
        .iter()
        .filter_map(|var| {
            if let Some((name, dt)) = get_var_name_type(var) {
                let enum_generic = enum_def.generics.clone();
                Some(quote! {
                    impl #enum_generic From<#dt> for #enum_name #enum_generic {
                        fn from(it: #dt) -> Self {
                            Self::#name(it)
                        }
                    }
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    quote! { #(#from_impls)* }
}
