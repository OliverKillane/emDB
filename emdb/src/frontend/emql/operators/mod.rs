//! Operators defined for the EMQL language
//! - Some special operators with differing syntax (such as let and use) are
//!   separate as their parsing & semantic analysis differ.
//! - Each operator is defined by its trait implementation.

use crate::frontend::emql::errors;
use combi::{
    core::{choice, mapsuc, nothing, recover, recursive, seq, seqdiff, setrepr, RecursiveHandle},
    derived::many0,
    logical::{not, or},
    macros::{choices, seqs},
    tokens::{
        basic::{
            collectuntil, getident, gettoken, isempty, matchident, matchpunct, peekident,
            peekpunct, recovgroup, syn, terminal,
        },
        derived::{listseptrailing, syntopunct},
        error::{embelisherr, error, expectederr},
        recovery::until,
        TokenDiagnostic, TokenIter, TokenParser,
    },
    Combi, CombiResult,
};
use proc_macro2::{Delimiter, Ident, Span};
use proc_macro_error::{Diagnostic, Level};
use std::collections::{HashMap, LinkedList};
use syn::Expr;

use crate::utils::misc::singlelist;

// translation for plans
use crate::plan::repr::*;

use super::ast::AstType;

// operators
mod op_update;
use op_update::*;
mod op_insert;
use op_insert::*;
mod op_delete;
use op_delete::*;
mod op_deref;
use op_deref::*;
mod op_ref;
use op_ref::*;
mod op_unique;
use op_unique::*;
mod op_use;
use op_use::*;
mod op_let;
use op_let::*;
mod op_return;
use op_return::*;
mod op_sort;
use op_sort::*;
mod op_map;
use op_map::*;
mod op_filter;
use op_filter::*;
mod op_row;
use op_row::*;
mod op_fold;
use op_fold::*;
mod op_assert;
use op_assert::*;
mod op_collect;
use op_collect::*;

// operator semantics tracking
pub(super) struct ReturnVal {
    pub span: Span,
    pub index: OpKey,
}

#[derive(Clone)]
pub(super) struct Continue {
    pub data_type: Record,
    pub prev_edge: EdgeKey,
    pub last_span: Span,
}

pub(super) enum StreamContext {
    Nothing { last_span: Span },
    Returned(ReturnVal),
    Continue(Continue),
}

pub(super) enum VarState {
    Used { created: Span, used: Span },
    Available { created: Span, state: Continue },
}

trait EMQLOperator: Sized {
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

// helper function for the functional style operators
fn functional_style<T>(name: &'static str, p: impl TokenParser<T>) -> impl TokenParser<(Ident, T)> {
    seq(matchident(name), recovgroup(Delimiter::Parenthesis, p))
}

fn fields_expr() -> impl TokenParser<Vec<(Ident, Expr)>> {
    listseptrailing(
        ',',
        mapsuc(
            seqs!(getident(), matchpunct('='), syntopunct(peekpunct(','))),
            |(id, (_, exp))| (id, exp),
        ),
    )
}

fn fields_assign() -> impl TokenParser<Vec<(Ident, (AstType, Expr))>> {
    listseptrailing(
        ',',
        mapsuc(
            seqs!(
                getident(),
                matchpunct(':'),
                type_parser('='),
                matchpunct('='),
                syntopunct(peekpunct(','))
            ),
            |(id, (_, (t, (_, e))))| (id, (t, e)),
        ),
    )
}

fn type_parser(punct: char) -> impl TokenParser<AstType> {
    choices! {
        peekident("ref") => mapsuc(seq(matchident("ref"), getident()), |(_, i)| AstType::TableRef(i)),
        otherwise => mapsuc(syntopunct(peekpunct(punct)), AstType::RsType)
    }
}

fn extract_fields<T>(
    fields: Vec<(Ident, T)>,
    err_fn: impl Fn(&Ident, &Ident) -> Diagnostic,
) -> (HashMap<Ident, T>, LinkedList<Diagnostic>) {
    let mut map_fields: HashMap<Ident, T> = HashMap::with_capacity(fields.len());
    let mut errors = LinkedList::new();
    for (id, content) in fields {
        if let Some((other_id, _)) = map_fields.get_key_value(&id) {
            errors.push_back(err_fn(&id, other_id));
        } else {
            map_fields.insert(id, content);
        }
    }

    (map_fields, errors)
}

// Boilerplate to connect operators (defined as structs) to the enums used to contain them in the ast and combi operators
// This will no longer be required once enum variant's are made first class types
// - See: [RFC 1450](https://github.com/rust-lang/rfcs/pull/1450) and [RFC 2593](https://github.com/rust-lang/rfcs/pull/2593)

#[derive(Debug)]
pub(crate) enum Operator {
    Return(Return),
    Let(Let),
    Use(Use),
    Update(Update),
    Insert(Insert),
    Delete(Delete),
    Map(Map),
    Unique(Unique),
    Filter(Filter),
    Row(Row),
    DeRef(DeRef),
    Sort(Sort),
    Fold(Fold),
    Assert(Assert),
    Collect(Collect),
}

pub fn parse_operator() -> impl TokenParser<Operator> {
    choices! {
        peekident(Return::NAME) => mapsuc(Return::build_parser(), Operator::Return),
        peekident(Let::NAME) => mapsuc(Let::build_parser(), Operator::Let),
        peekident(Use::NAME) => mapsuc(Use::build_parser(), Operator::Use),
        peekident(Update::NAME) => mapsuc(Update::build_parser(), Operator::Update),
        peekident(Insert::NAME) => mapsuc(Insert::build_parser(), Operator::Insert),
        peekident(Delete::NAME) => mapsuc(Delete::build_parser(), Operator::Delete),
        peekident(Map::NAME) => mapsuc(Map::build_parser(), Operator::Map),
        peekident(Unique::NAME) => mapsuc(Unique::build_parser(), Operator::Unique),
        peekident(Filter::NAME) => mapsuc(Filter::build_parser(), Operator::Filter),
        peekident(Row::NAME) => mapsuc(Row::build_parser(), Operator::Row),
        peekident(DeRef::NAME) => mapsuc(DeRef::build_parser(), Operator::DeRef),
        peekident(Sort::NAME) => mapsuc(Sort::build_parser(), Operator::Sort),
        peekident(Fold::NAME) => mapsuc(Fold::build_parser(), Operator::Fold),
        peekident(Assert::NAME) => mapsuc(Assert::build_parser(), Operator::Assert),
        peekident(Collect::NAME) => mapsuc(Collect::build_parser(), Operator::Collect),
        otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("expected an operator but got {}", t)))
    }
}

pub fn build_logical(
    op: Operator,
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    cont: Option<Continue>,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    match op {
        Operator::Return(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Let(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Use(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Update(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Insert(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Delete(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Map(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Unique(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Filter(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Row(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::DeRef(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Sort(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Fold(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Assert(i) => i.build_logical(lp, tn, qk, vs, cont),
        Operator::Collect(i) => i.build_logical(lp, tn, qk, vs, cont),
    }
}
