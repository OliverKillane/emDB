//! ## EMQL Operators
//! each operator defines its parser, and how it is translated into the logical plan.
//!
//! Every operator has:
//! - A a module representing the operator, with a struct included.
//! - Each needs to implement [`EMQLOperator`]   
//! - the [`create_operator`] macro generates a single enumeration and
//!   associated parse and logical translation functions (we avoid using
//!   `Box<dyn EMQLOperator>` by polymorphism through the [`Operator`] enum)
//!
//! To create a new operator, simply add a new module and [`EMQLOperator`], then
//! add it to the [`create_operator`] macro invocation.

use super::ast::AstType;
use crate::frontend::emql::errors;
use crate::frontend::emql::parse::{
    fields_assign, fields_expr, functional_style, type_parser_to_punct,
};
use crate::frontend::emql::sem::{
    ast_typeto_scalar, create_scanref, extract_fields, generate_access, get_all_cols,
    linear_builder, update_incomplete, valid_linear_builder, Continue, LinearBuilderState,
    ReturnVal, StreamContext, VarState,
};
use crate::plan;
use crate::utils::misc::{result_to_opt, singlelist};
use combi::{
    core::{choice, mapsuc, seq, setrepr},
    macros::{choices, seqs},
    tokens::{
        basic::{
            collectuntil, getident, gettoken, isempty, matchident, matchpunct, peekident,
            peekpunct, syn,
        },
        derived::{listseptrailing, syntopunct},
        error::{error, expectederr},
        TokenParser,
    },
};
use proc_macro2::{Ident, Span};
use proc_macro_error::{Diagnostic, Level};
use std::{
    collections::{HashMap, LinkedList},
    fmt::Debug,
};
use syn::Expr;

trait EMQLOperator: Sized + Debug {
    const NAME: &'static str;

    fn build_parser() -> impl TokenParser<Self>;

    /// Convert the operator to a logical plan node
    /// - `tn` represents the identifier to table mapping
    #[allow(clippy::too_many_arguments)]
    fn build_logical(
        self,
        lp: &mut plan::Plan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        op_ctx: plan::Key<plan::Context>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>>;
}

// Boilerplate to connect operators (defined as structs) to the enums used to contain them in the ast and combi operators
// This will no longer be required once enum variant's are made first class types
// - See: [RFC 1450](https://github.com/rust-lang/rfcs/pull/1450) and [RFC 2593](https://github.com/rust-lang/rfcs/pull/2593)
//
// Could use `enumtrait`, but here we also want to generate parse_operator, which has a pattern enum_trait cannot generate.
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
                    peekident($t::NAME) => expectederr(mapsuc($t::build_parser(), $op::$t)),
                )*
                otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("expected an operator but got {}", t)))
            }
        }

        #[allow(clippy::too_many_arguments)]
        pub fn build_logical(
            op: $op,
            lp: &mut plan::Plan,
            tn: &HashMap<Ident, plan::Key<plan::Table>>,
            qk: plan::Key<plan::Query>,
            vs: &mut HashMap<Ident, VarState>,
            ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
            op_ctx: plan::Key<plan::Context>,
            cont: Option<Continue>,
        ) -> Result<StreamContext, LinkedList<Diagnostic>> {
            match op {
                $(
                    $op::$t(i) => i.build_logical(lp, tn, qk, vs, ts, op_ctx, cont),
                )*
            }
        }
    };
}

// Import to make available to private docs
#[allow(unused_imports)]
pub(crate) use create_operator;

create_operator!(
    Operator as op_return::Return,
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
    op_collect::Collect,
    op_take::Take,
    op_fork::Fork,
    op_union::Union
);
