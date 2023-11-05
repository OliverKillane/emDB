#![allow(dead_code)]
#![allow(unused_variables)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{diagnostic, proc_macro_error, Diagnostic, Level};
extern crate proc_macro;

mod backend;
mod frontend;
mod plan;

use crate::backend::{Backend, Volcano};
use crate::frontend::{Frontend, EMQL};

#[proc_macro_error]
#[proc_macro]
pub fn database(tk: TokenStream) -> TokenStream {
    match EMQL::from_tokens(TokenStream2::from(tk)) {
        Err(ds) => {
            ds.into_iter().for_each(Diagnostic::emit);
            TokenStream::new()
        }
        Ok(lp) => proc_macro::TokenStream::from(Volcano::generate_code(lp)),
    }
}

#[proc_macro_error]
#[proc_macro]
pub fn test_macro(tk: TokenStream) -> TokenStream {
    let x = TokenStream2::from(tk);

    for t in x {
        // match t {
        //     proc_macro2::TokenTree::Group(_) => todo!(),
        //     proc_macro2::TokenTree::Ident(_) => todo!(),
        //     proc_macro2::TokenTree::Punct(_) => todo!(),
        //     proc_macro2::TokenTree::Literal(_) => todo!(),
        // }
        Diagnostic::spanned(t.span(), Level::Error, format!("{t}")).emit()
    }

    TokenStream::new()
}
