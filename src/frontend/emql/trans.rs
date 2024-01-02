use std::collections::LinkedList;

use proc_macro_error::Diagnostic;

use crate::{frontend::emql::ast::AST, plan::repr::LogicalPlan};

pub(super) fn translate<'a>(ast: AST) -> Result<LogicalPlan<'a>, LinkedList<Diagnostic>> {
    todo!()
}
