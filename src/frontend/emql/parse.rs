use super::ast::AST;
use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;

pub(super) fn parse(ts: TokenStream) -> Result<AST, Vec<Diagnostic>> {
    todo!()
}
