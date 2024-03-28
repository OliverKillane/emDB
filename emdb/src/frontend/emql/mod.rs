//! # The emQL language frontend
//! [TokenStream] -> parse -> ([AST] + errors) -> translate -> ([LogicalPlan] + errors) -> [Backend]
mod ast;
mod errors;
mod operators;
mod parse;
mod sem;
use std::collections::LinkedList;

use crate::{
    frontend::Frontend,
    plan::{repr::LogicalPlan, targets::Targets},
};
use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;

pub struct Emql;

impl Frontend for Emql {
    fn from_tokens(input: TokenStream) -> Result<(Targets, LogicalPlan), LinkedList<Diagnostic>> {
        let ast = parse::parse(input)?;
        let lp = sem::ast_to_logical(ast)?;

        Err(LinkedList::new())
        // let (targets, res_ast) = trans::translate(ast)?;
        // Ok((targets, res_ast))
    }
}
