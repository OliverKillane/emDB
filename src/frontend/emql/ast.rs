// TODO: parameterize by span type (want to go to resolved AST spanned by types)
use proc_macro2::Span;
use syn;

pub(super) enum SingleType {
    RsType(syn::Type),
}

pub(super) enum ChainOperator {
    ValBack,
    RefSend,
    ValSend,
}

pub(super) struct Condition {}

pub(super) struct Constraint {}

pub(super) enum BinaryOperator {
    Cross,
    Join(Condition),
    Union,
    Minus,
}

pub(super) struct Spanned<T> {
    pub data: T,
    pub span: Span,
}
do!()
pub(super) enum SingleExpr {
    RsExpr(syn::Expr),
}

pub(super) enum StreamExpr {
    Return,
    Ident(Spanned<String>),
    Let(Spanned<String>),
    Operator {
        name: Spanned<String>,
        args: Vec<()>, //TODO
    },
    UnaryOperator {
        op: Spanned<ChainOperator>,
        successor: Box<Spanned<StreamExpr>>,
    },
    BinaryOperator {
        left: Box<Spanned<StreamExpr>>,
        op: Spanned<BinaryOperator>,
    },
}

pub(super) struct Table {
    pub name: Spanned<String>,
    pub cols: Vec<(Spanned<String>, Spanned<SingleType>)>,
    pub cons: Vec<Spanned<Constraint>>,
}

pub(super) struct Query {
    pub name: String,
    pub params: Vec<(String, syn::Type)>,
    pub streams: Vec<StreamExpr>,
}

pub(super) struct AST {
    pub name: Spanned<String>,
    pub tables: Vec<Table>,
    pub queries: Vec<Query>,
}
