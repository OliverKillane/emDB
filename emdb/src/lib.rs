// #![warn(clippy::pedantic)]
// #![allow(clippy::linkedlist)]
// linked lists used for quick merging of errors lists, and are only iterated over for fast-escape failure case
#![allow(dead_code)]
#![allow(unused_variables)]
use std::collections::LinkedList;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::proc_macro_error;
use quote::quote;

extern crate proc_macro;

mod backend;
mod frontend;
mod plan;
mod utils;

fn make_impl<F: frontend::Frontend>(tk: TokenStream) -> TokenStream {
    match F::from_tokens(TokenStream2::from(tk)) {
        Err(ds) => {
            for d in ds {
                d.emit();
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
                e.emit();
            }

            proc_macro::TokenStream::from(quote! {
                #(#impls)*
            })
        }
    }
}

macro_rules! create_frontend {
    ($frontend:ident as $implement:path => $($t:tt)*) => {
        $($t)*
        #[proc_macro_error]
        #[proc_macro]
        pub fn $frontend(tk: TokenStream) -> TokenStream {
            make_impl::<$implement>(tk)
        }
    };
}

create_frontend!(emql as frontend::Emql =>
    /// The `emql` language frontend for [emdb](crate).
);
