//! # The emQL language frontend
//! [`TokenStream`] -> parse -> ([AST] + errors) -> translate -> ([`LogicalPlan`] + errors) -> [Backend]
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
