//! # The emQL language frontend
//! [TokenStream] -> parse -> ([AST] + errors) -> translate -> ([LogicalPlan] + errors) -> [Backend]
mod ast;
mod parse;
mod sem;
mod trans;
mod trans2;
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

        Err(LinkedList::new())

        // let (targets, res_ast) = trans::translate(ast)?;
        // Ok((targets, res_ast))
    }
}
