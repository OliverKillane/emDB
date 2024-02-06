#![allow(dead_code)]
#![allow(unused_variables)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{proc_macro_error, Diagnostic, Level};
use quote::quote;
extern crate proc_macro;

mod backend;
mod frontend;
mod plan;
mod utils;

use crate::backend::{Backend, GraphViz, Simple};
use crate::frontend::{Emql, Frontend};

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
        Ok((targets, lp)) => {
            let impls = targets
                .backends
                .into_iter()
                .map(|(t, backend)| {
                    let code = (match backend {
                        plan::targets::Target::Simple => Simple::generate_code,
                        plan::targets::Target::Graphviz => GraphViz::generate_code,
                    })(&lp);

                    quote! {
                        mod #t {
                            #code
                        }
                    }
                })
                .collect::<Vec<_>>();

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
