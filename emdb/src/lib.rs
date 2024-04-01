#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::LinkedList;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{proc_macro_error, Diagnostic, Level};
use quote::quote;
extern crate proc_macro;

mod backend;
mod frontend;
mod plan;
mod utils;

use frontend::{Emql, Frontend};

#[proc_macro_error]
#[proc_macro]
pub fn database(tk: TokenStream) -> TokenStream {
    match Emql::from_tokens(TokenStream2::from(tk)) {
        Err(ds) => {
            for d in ds {
                d.emit()
            }
            TokenStream::new()
        }
        Ok((lp, bks)) => {
            let mut errors = LinkedList::new();
            let impls = bks
                .impls
                .into_iter()
                .filter_map(|(id, backend)| {
                    match backend::generate_code(backend, id.clone(), &lp) {
                        Ok(code) => Some(quote! {
                            mod #id {
                                #code
                            }
                        }),
                        Err(mut e) => {
                            errors.append(&mut e);
                            None
                        }
                    }
                })
                .collect::<Vec<_>>();

            for e in errors {
                e.emit()
            }

            proc_macro::TokenStream::from(quote! {
                #(#impls)*
            })
        }
    }
}

#[proc_macro_error]
#[proc_macro]
pub fn bob(tk: TokenStream) -> TokenStream {
    Diagnostic::new(Level::Error, String::from("bob")).emit();
    TokenStream::new()
}
