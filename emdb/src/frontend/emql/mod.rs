//! # The emQL language frontend
//! [TokenStream] -> parse -> ([AST] + errors) -> translate -> ([LogicalPlan] + errors) -> [Backend]
mod ast;
mod parse;
mod sem;
mod trans;
use std::collections::LinkedList;

use crate::{frontend::Frontend, plan::repr::LogicalPlan};
use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;

pub struct Emql;

impl<'a> Frontend<'a> for Emql {
    fn from_tokens(input: TokenStream) -> Result<LogicalPlan<'a>, LinkedList<Diagnostic>> {
        let ast = parse::parse(input)?;
        let res_ast = trans::translate(ast)?;
        Ok(res_ast)
    }
}
