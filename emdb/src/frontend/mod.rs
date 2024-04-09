//! # [emDB](crate) frontends
//! 
//! ## What is a frontend?
//! 
//! ## [Diagnostics API](proc_macro_error::Diagnostic)
//! 

mod emql;
mod boss;
mod sql;

use std::collections::LinkedList;

use crate::backend;
use crate::plan;

pub use emql::Emql;

use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;

pub struct Diagnostics {
    errs: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self { errs: Vec::new() }
    }
    pub fn add(&mut self, d: Diagnostic) {
        self.errs.push(d);
    }
    pub fn emit(self) {
        self.errs.into_iter().for_each(Diagnostic::emit);
    }

    pub fn empty(&self) -> bool {
        self.errs.is_empty()
    }
}

pub trait Frontend {
    fn from_tokens(
        input: TokenStream,
    ) -> Result<(plan::Plan, backend::Targets), LinkedList<Diagnostic>>;
}
