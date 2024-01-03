mod emql;
mod sql;

use std::collections::LinkedList;

use crate::plan::repr::LogicalPlan;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::Diagnostic;

pub(crate) use emql::Emql;

pub(crate) struct Diagnostics {
    errs: Vec<Diagnostic>,
}

impl Diagnostics {
    pub(crate) fn new() -> Self {
        Self { errs: Vec::new() }
    }
    pub(crate) fn add(&mut self, d: Diagnostic) {
        self.errs.push(d)
    }
    pub(crate) fn emit(self) {
        self.errs.into_iter().for_each(Diagnostic::emit)
    }

    pub(crate) fn empty(&self) -> bool {
        self.errs.is_empty()
    }
}

pub(crate) trait Frontend<'a> {
    fn from_tokens(input: TokenStream2) -> Result<LogicalPlan<'a>, LinkedList<Diagnostic>>;
}
