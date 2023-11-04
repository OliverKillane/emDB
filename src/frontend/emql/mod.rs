//! # The emQL language frontend
mod ast;
mod parse;
mod sem;
mod trans;

use crate::{frontend::Frontend, plan::repr::LogicalPlan};
use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;
pub struct EMQL;

impl<'a> Frontend<'a> for EMQL {
    fn from_tokens(input: TokenStream) -> Result<LogicalPlan<'a>, Vec<Diagnostic>> {
        let ast = parse::parse(input)?;
        let resolved_ast = sem::semantic_analysis(ast)?;
        Ok(trans::translate(resolved_ast))
    }
}
