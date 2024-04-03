use crate::plan;
use crate::utils::misc::singlelist;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use std::collections::{HashMap, LinkedList};

pub trait EMDBBackend: Sized {
    const NAME: &'static str;
    fn parse_options(options: Option<TokenStream>) -> Result<Self, LinkedList<Diagnostic>>;
    fn generate_code(
        self,
        impl_name: Ident,
        plan: &plan::Plan,
    ) -> Result<TokenStream, LinkedList<Diagnostic>>;
}

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
                    $t::NAME => $t::parse_options(options).map(|v| $op::$t(v)),
                )*
                _ => Err(singlelist(no_such_backend(&backend_name)))
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

create_backend!(Backend as planviz::PlanViz);

pub struct Targets {
    pub impls: HashMap<Ident, Backend>,
}

fn no_such_backend(backend_name: &Ident) -> Diagnostic {
    Diagnostic::spanned(
        backend_name.span(),
        Level::Error,
        "No such backend".to_string(),
    )
}
