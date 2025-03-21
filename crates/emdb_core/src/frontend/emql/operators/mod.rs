//! ## EMQL Operators
//! each operator defines its parser, and how it is translated into the logical plan.
//!
//! Every operator has:
//! - A a module representing the operator, with a struct included.
//! - Each needs to implement [`EMQLOperator`]   
//! - the [`create_operator`] macro generates a single enumeration and
//!   associated parse and logical translation functions (we avoid using
//!   `Box<dyn EMQLOperator>` by polymorphism through the [`Operator`] enum)
//!   (similar to [enumtrait]).
//!
//! To create a new operator, simply add a new module and [`EMQLOperator`], then
//! add it to the [`create_operator`] macro invocation.

use super::ast::{AstType, StreamExpr};
use crate::frontend::emql::errors;
use crate::frontend::emql::parse::{
    fields_assign, fields_expr, functional_style, type_parser_to_punct, ContextRecurHandle,
};
use crate::frontend::emql::sem::{
    add_streams_to_context, assign_new_var, check_fields_type, create_scanref, discard_ends,
    extract_fields_ordered, generate_access, get_all_cols, get_user_fields, linear_builder,
    query_ast_typeto_scalar, update_incomplete, valid_linear_builder, Continue, FieldComparison,
    LinearBuilderState, ReturnVal, StreamContext, VarState,
};
use crate::plan;
use crate::utils::misc::{result_to_opt, singlelist};
use combi::{
    core::{choice, mapsuc, nothing, seq, setrepr},
    macros::{choices, seqs},
    tokens::{
        basic::{
            collectuntil, getident, gettoken, isempty, matchident, matchpunct, peekident,
            peekpunct, recovgroup, syn,
        },
        derived::{listseptrailing, syntopunct},
        error::{error, expectederr},
        TokenParser,
    },
};
use proc_macro2::{Delimiter, Ident, Span};
use proc_macro_error2::{Diagnostic, Level};
use std::{
    collections::{HashMap, LinkedList},
    fmt::Debug,
};
use syn::Expr;

trait EMQLOperator: Sized + Debug {
    const NAME: &'static str;

    /// Parse the operator's tokens (taken directly from the stream, after peeking for [`EMQLOperator::NAME`])
    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self>;

    /// Convert the operator to a logical plan node
    /// - Needs to ensure a valid plan is left even on logical errors (to allow
    ///   other streams, inner contexts to be analysed).
    #[allow(clippy::too_many_arguments)]
    fn build_logical(
        self,
        lp: &mut plan::Plan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
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

        pub fn parse_operator(ctx_recur: ContextRecurHandle) -> impl TokenParser<$op> {
            choices! {
                $(
                    peekident($t::NAME) => expectederr(mapsuc($t::build_parser(ctx_recur.clone()), $op::$t)),
                )*
                otherwise => error(gettoken, |t|
                    Diagnostic::spanned(t.span(), Level::Error, format!("expected an operator but got {t}"))
                        .help(format!("Availabe operators are {}", [$($t::NAME,)*].join(", ")))
                )
            }
        }

        #[allow(clippy::too_many_arguments)]
        pub fn build_logical(
            op: $op,
            lp: &mut plan::Plan,
            tn: &HashMap<Ident, plan::Key<plan::Table>>,
            vs: &mut HashMap<Ident, VarState>,
            ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
            op_ctx: plan::Key<plan::Context>,
            cont: Option<Continue>,
        ) -> Result<StreamContext, LinkedList<Diagnostic>> {
            match op {
                $(
                    $op::$t(i) => i.build_logical(lp, tn, vs, ts, op_ctx, cont),
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
    op_union::Union,
    op_lift::Lift,
    op_groupby::GroupBy,
    op_join::Join,
    op_combine::Combine,
    op_count::Count
);
