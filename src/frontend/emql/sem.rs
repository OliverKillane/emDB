use proc_macro_error::Diagnostic;

use crate::frontend::emql::ast::AST;

pub(super) fn semantic_analysis(ast: AST) -> Result<AST, Vec<Diagnostic>> {
    todo!()
}
