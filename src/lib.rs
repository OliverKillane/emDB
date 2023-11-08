#![allow(dead_code)]
#![allow(unused_variables)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{proc_macro_error, Diagnostic, Level};
extern crate proc_macro;

mod backend;
mod frontend;
mod plan;
mod utils;

use crate::backend::{Backend, Volcano};
use crate::frontend::{Frontend, EMQL};

#[proc_macro_error]
#[proc_macro]
pub fn database(tk: TokenStream) -> TokenStream {
    match EMQL::from_tokens(TokenStream2::from(tk)) {
        Err(ds) => {
            ds.emit();
            TokenStream::new()
        }
        Ok(lp) => proc_macro::TokenStream::from(Volcano::generate_code(lp)),
    }
}

#[proc_macro_error]
#[proc_macro]
pub fn bob(tk: TokenStream) -> TokenStream {
    Diagnostic::new(Level::Error, String::from("bob")).emit();
    TokenStream::new()
}
