mod emql;

// TODO: add some basic implementation
mod boss;
mod sql;

use std::collections::LinkedList;

use crate::backend;
use crate::plan;

pub(crate) use emql::Emql;

use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;

pub(crate) struct Diagnostics {
    errs: Vec<Diagnostic>,
}

impl Diagnostics {
    pub(crate) fn new() -> Self {
        Self { errs: Vec::new() }
    }
    pub(crate) fn add(&mut self, d: Diagnostic) {
        self.errs.push(d);
    }
    pub(crate) fn emit(self) {
        self.errs.into_iter().for_each(Diagnostic::emit);
    }

    pub(crate) fn empty(&self) -> bool {
        self.errs.is_empty()
    }
}

pub(crate) trait Frontend {
    fn from_tokens(
        input: TokenStream,
    ) -> Result<(plan::Plan, backend::Targets), LinkedList<Diagnostic>>;
}
