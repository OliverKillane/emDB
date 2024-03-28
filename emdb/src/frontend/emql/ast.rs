// TODO: parameterize by span type (want to go to resolved AST spanned by types)
use super::operators::Operator;
use proc_macro2::{Ident, Span};
use syn::{Expr, Type};

#[derive(Debug)]
pub(super) enum AstType {
    RsType(syn::Type),
    TableRef(Ident),
}

#[derive(Debug)]
pub(super) struct Connector {
    /// single (~>) or stream (|>)
    pub stream: bool,
    pub span: Span,
}

#[derive(Debug)]
pub(super) struct StreamExpr {
    pub op: Operator,
    pub con: Option<(Connector, Box<StreamExpr>)>,
}

#[derive(Debug)]
pub(super) enum ConstraintExpr {
    Unique { field: Ident },
    Pred(Expr),
    Limit { size: Expr },
}

#[derive(Debug)]
pub(super) struct Constraint {
    pub alias: Option<Ident>,
    pub method_span: Span,
    pub expr: ConstraintExpr,
}

#[derive(Debug)]
pub(super) struct Table {
    pub name: Ident,
    pub cols: Vec<(Ident, Type)>,
    pub cons: Vec<Constraint>,
}

#[derive(Debug)]
pub(super) struct Query {
    pub name: Ident,
    pub params: Vec<(Ident, AstType)>,
    pub streams: Vec<StreamExpr>,
}

#[derive(Debug)]
pub(super) struct BackendImpl {
    pub name: Ident,
    pub target: BackendKind,
}

#[derive(Debug)]
pub(super) enum BackendKind {
    Graph,
    Simple,
}

#[derive(Debug)]
pub(super) struct Ast {
    pub backends: Vec<BackendImpl>,
    pub tables: Vec<Table>,
    pub queries: Vec<Query>,
}
