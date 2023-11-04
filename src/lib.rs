#![allow(dead_code)]
#![allow(unused_variables)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::Diagnostic;
extern crate proc_macro;

mod backend;
mod frontend;
mod plan;

use crate::backend::{Backend, Volcano};
use crate::frontend::{Frontend, EMQL};

#[proc_macro]
pub fn database(tk: TokenStream) -> TokenStream {
    match EMQL::from_tokens(TokenStream2::from(tk)) {
        Err(diags) => {
            diags.into_iter().for_each(Diagnostic::emit);
            TokenStream::new()
        }
        Ok(lp) => proc_macro::TokenStream::from(Volcano::generate_code(lp)),
    }
}
