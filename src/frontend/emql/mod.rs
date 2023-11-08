//! # The emQL language frontend
//! [TokenStream] -> parse -> ([AST] + errors) -> translate -> ([LogicalPlan] + errors) -> [Backend]
mod ast;
mod combs;
mod parse;
mod sem;
mod trans;
use crate::{frontend::Frontend, plan::repr::LogicalPlan};
use proc_macro2::TokenStream;

use super::Diagnostics;
pub struct EMQL;

impl<'a> Frontend<'a> for EMQL {
    fn from_tokens(input: TokenStream) -> Result<LogicalPlan<'a>, Diagnostics> {
        let mut errs = Diagnostics::new();
        if let Some(ast) = parse::parse(input, &mut errs) {
            if let Some(lp) = trans::translate(ast, &mut errs) {
                if errs.empty() {
                    return Ok(lp);
                }
            }
        }
        Err(errs)
    }
}
