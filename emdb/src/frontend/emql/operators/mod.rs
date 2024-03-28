//! ## EMQL Operators
//! each operator defines its parser, and how it is translated into the logical plan.
//! 
//! Every operator has:
//! - A a module representing the operator, with a struct included.
//! - Each needs to implement [EMQLOperator]   
//! - the [create_operator] macro generates a single enumeration and 
//!   associated parse and logical translation functions (we avoid using 
//!   `Bx<dyn EMQLOperator>` by polymorphism through the [Operator] enum)
//! 
//! To create a new operator, simply add a new module and [EMQLOperator], then 
//! add it to the [create_operator] macro invocation.

use crate::frontend::emql::errors;
use combi::{
    core::{choice, mapsuc, nothing, seq},
    macros::{choices, seqs},
    tokens::{
        basic::{
            collectuntil, getident, gettoken, isempty, matchident, matchpunct, peekident,
            recovgroup, syn,
        },
        derived::listseptrailing,
        error::error,
        TokenParser,
    },
};
use proc_macro2::{Delimiter, Ident, Span};
use proc_macro_error::{Diagnostic, Level};
use std::{
    collections::{HashMap, LinkedList},
    fmt::Debug,
};
use syn::Expr;

use crate::utils::misc::singlelist;

// translation for plans
use super::ast::AstType;
use crate::frontend::emql::parse::{
    fields_assign, fields_expr, functional_style,
};
use crate::frontend::emql::sem::{extract_fields, Continue, ReturnVal, StreamContext, VarState};
use crate::plan::repr::*;

trait EMQLOperator: Sized + Debug {
    const NAME: &'static str;

    fn build_parser() -> impl TokenParser<Self>;

    /// Convert the operator to a logical plan node
    fn build_logical(
        self,
        lp: &mut LogicalPlan,
        tn: &HashMap<Ident, TableKey>,
        qk: QueryKey,
        vs: &mut HashMap<Ident, VarState>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>>;
}

// Boilerplate to connect operators (defined as structs) to the enums used to contain them in the ast and combi operators
// This will no longer be required once enum variant's are made first class types
// - See: [RFC 1450](https://github.com/rust-lang/rfcs/pull/1450) and [RFC 2593](https://github.com/rust-lang/rfcs/pull/2593)

macro_rules! create_operator {
    ($op:ident as $($m:ident :: $t:ident),*) => {

        $(
            mod $m;
            use $m::$t;
        )*

        #[derive(Debug)]
        pub(crate) enum $op {
            $(
                $t($t),
            )*
        }

        pub fn parse_operator() -> impl TokenParser<$op> {
            choices! {
                $(
                    peekident($t::NAME) => mapsuc($t::build_parser(), $op::$t),
                )*
                otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("expected an operator but got {}", t)))
            }
        }

        pub fn build_logical(
            op: $op,
            lp: &mut LogicalPlan,
            tn: &HashMap<Ident, TableKey>,
            qk: QueryKey,
            vs: &mut HashMap<Ident, VarState>,
            cont: Option<Continue>,
        ) -> Result<StreamContext, LinkedList<Diagnostic>> {
            match op {
                $(
                    $op::$t(i) => i.build_logical(lp, tn, qk, vs, cont),
                )*
            }
        }
    };
}

create_operator!(Operator as
    op_return::Return, 
    op_ref::Ref, 
    op_let::Let, 
    op_use::Use, 
    op_update::Update, 
    op_insert::Insert, 
    op_delete::Delete, 
    op_map::Map, 
    op_unique::Unique, 
    op_filter::Filter, 
    op_row::Row, 
    op_deref::DeRef, 
    op_sort::Sort, 
    op_fold::Fold, 
    op_assert::Assert,
    op_collect::Collect
);
