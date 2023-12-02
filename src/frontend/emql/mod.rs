//! # The emQL language frontend
//! [TokenStream] -> parse -> ([AST] + errors) -> translate -> ([LogicalPlan] + errors) -> [Backend]
mod ast;
mod combs;
mod parse;

// TODO: replace
mod parse2;

mod sem;
mod trans;
use std::collections::LinkedList;

use crate::{frontend::Frontend, plan::repr::LogicalPlan};
use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;

use self::parse2::parse;

use super::Diagnostics;
pub struct EMQL;

impl<'a> Frontend<'a> for EMQL {
    fn from_tokens(input: TokenStream) -> Result<LogicalPlan<'a>, LinkedList<Diagnostic>> {
        let ast = parse2::parse(input)?;
        let res_ast = trans::translate(ast)?;
        Ok(res_ast)
    }
}
