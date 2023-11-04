//! # The emQL language frontend
mod ast;
mod parse;

use crate::{frontend::Frontend, plan::repr::LogicalPlan};
use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;
struct EMQL;

impl<'a> Frontend<'a> for EMQL {
    fn from_tokens(input: TokenStream) -> Result<LogicalPlan<'a>, Vec<Diagnostic>> {
        todo!()
    }
}
