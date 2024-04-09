//! # The emQL language frontend
//! ## What is emQL
//! 
//! ## emQL over SQL?
//! 
//! ## Implementation 
//! 
//! ### [rustc API](https://rustc-dev-guide.rust-lang.org/rustc-driver.html) versus token passthrough
//! 
//! ### [Token](proc_macro2::TokenStream) parsing
//! 

mod ast;
mod errors;
mod operators;
mod parse;
mod sem;
use crate::backend;
use std::collections::LinkedList;

use crate::{frontend::Frontend, plan};
use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;

pub struct Emql;

impl Frontend for Emql {
    fn from_tokens(
        input: TokenStream,
    ) -> Result<(plan::Plan, backend::Targets), LinkedList<Diagnostic>> {
        sem::ast_to_logical(parse::parse(input)?)
    }
}
