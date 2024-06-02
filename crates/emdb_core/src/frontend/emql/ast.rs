//! # emQL Abstract Syntax Tree
//! A basic AST, without error nodes. Includes spans required for [`super::sem`] to create [`super::errors`].

use super::operators::Operator;
use proc_macro2::{Ident, Span, TokenStream};
use syn::{Expr, Type};

#[derive(Debug)]
pub(super) enum AstType {
    RsType(syn::Type),
    TableRef(Ident),
    Custom(Ident),
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
    pub alias: Ident,
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
pub struct Context {
    pub params: Vec<(Ident, AstType)>,
    pub streams: Vec<StreamExpr>,
}

#[derive(Debug)]
pub(super) struct Query {
    pub name: Ident,
    pub context: Context,
}

#[derive(Debug)]
pub(super) struct BackendImpl {
    pub impl_name: Ident,
    pub backend_name: Ident,
    pub options: Option<TokenStream>,
}

#[derive(Debug)]
pub(super) struct Ast {
    pub backends: Vec<BackendImpl>,
    pub tables: Vec<Table>,
    pub queries: Vec<Query>,
}
