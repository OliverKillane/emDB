// TODO: parameterize by span type (want to go to resolved AST spanned by types)
use proc_macro2::Span;
use syn;

pub(super) enum SingleType {
    RsType(syn::Type),
}

pub(super) enum UnaryOperator {
    ValBack,
    RefSend,
    ValSend,
}

pub(super) struct Condition {}

pub(super) enum BinaryOperator {
    Cross,
    Join(Condition),
    Union,
    Minus,
}

pub(super) struct Spanned<T> {
    data: T,
    span: Span,
}

pub(super) enum Stream {
    Return,
    Ident(Spanned<String>),
    Let(Spanned<String>),
    UnaryOperator {
        op: Spanned<UnaryOperator>,
        successor: Box<Spanned<Stream>>,
    },
    BinaryOperator {
        left: Box<Spanned<Stream>>,
        op: Spanned<BinaryOperator>,
        right: Box<Spanned<Stream>>,
    },
}

pub(super) struct Table {
    name: String,
    a: SingleType,
}

pub(super) struct Query {
    name: String,
    params: Vec<(String, SingleType)>,
    streams: Vec<Stream>,
}

pub(super) struct AST {
    name: String,
    tables: Vec<Table>,
    queries: Vec<Query>,
}
