use proc_macro_error::{Diagnostic, Level};
use proc_macro2::Span;
use super::{EMDBBackend, PlanViz};

pub fn no_accepted_options(s: Span) -> Diagnostic {
    Diagnostic::spanned(s, Level::Error, format!("No options are accepted for the {} backend.", PlanViz::NAME))
}