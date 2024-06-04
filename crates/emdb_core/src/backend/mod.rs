//! # emDB Backends
//! Each bakend takes an immutable reference to a plan, and generates artifacts
//! from this (code, files)

use crate::{plan, utils::misc::singlelist};

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use std::collections::{HashMap, LinkedList};

pub trait EMDBBackend: Sized {
    /// The name of the backend to be invoked by the user.
    const NAME: &'static str;

    /// Each backend can optionally take arbitrary tokens as configuration.
    fn parse_options(
        backend_name: &Ident,
        options: Option<TokenStream>,
    ) -> Result<Self, LinkedList<Diagnostic>>;

    /// Generate the code to be expanded in-place. Any extra file access/IO
    /// should also be done here.
    fn generate_code(
        self,
        impl_name: Ident,
        plan: &plan::Plan,
    ) -> Result<TokenStream, LinkedList<Diagnostic>>;
}

/// Generate the functions for parsing, generating code
macro_rules! create_backend {
    ($op:ident as $($m:ident :: $t:ident),*) => {

        $(
            mod $m;
            use $m::$t;
        )*

        pub enum $op {
            $(
                $t($t),
            )*
        }

        pub fn parse_options(backend_name: Ident, options: Option<TokenStream>) -> Result<$op, LinkedList<Diagnostic>> {
            match backend_name.to_string().as_str() {
                $(
                    $t::NAME => $t::parse_options(&backend_name, options).map(|v| $op::$t(v)),
                )*
                _ => Err(singlelist(Diagnostic::spanned(
                    backend_name.span(),
                    Level::Error,
                    format!("No such backend `{backend_name}`"),
                ).help(format!("Available backends are: {}", [$($t::NAME,)*].join(", ")))))
            }
        }

        pub fn generate_code(
            op: $op,
            impl_name: Ident,
            plan: &plan::Plan
        ) -> Result<TokenStream, LinkedList<Diagnostic>> {
            match op {
                $(
                    $op::$t(i) => i.generate_code(impl_name, plan),
                )*
            }
        }
    };
}

create_backend!(Backend as planviz::PlanViz, serialized::Serialized);

/// Wrapper for the targets to produce
pub struct Targets {
    pub impls: HashMap<Ident, Backend>,
}
