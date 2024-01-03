use std::collections::LinkedList;

use proc_macro_error::Diagnostic;

use crate::{frontend::emql::ast::Ast, plan::repr::LogicalPlan};

pub(super) fn translate<'a>(ast: Ast) -> Result<LogicalPlan<'a>, LinkedList<Diagnostic>> {
    todo!()
}
