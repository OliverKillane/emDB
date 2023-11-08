use crate::{
    frontend::{emql::ast::AST, Diagnostics},
    plan::repr::LogicalPlan,
};

pub(super) fn translate<'a>(ast: AST, errs: &mut Diagnostics) -> Option<LogicalPlan<'a>> {
    todo!()
}
