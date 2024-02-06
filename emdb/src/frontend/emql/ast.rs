// TODO: parameterize by span type (want to go to resolved AST spanned by types)
use proc_macro2::{Ident, Span};
use syn::{self, Expr, Type};

#[derive(Debug)]
pub(super) enum Operator {
    Ret { ret_span: Span },
    Ref { ref_span: Span, table_name: Ident },
    Let { let_span: Span, var_name: Ident },
    Use { use_span: Span, var_name: Ident },
    FuncOp { fn_span: Span, op: FuncOp },
}

#[derive(Debug)]
pub(super) enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug)]
pub(super) enum FuncOp {
    Update {
        reference: Ident,
        fields: Vec<(Ident, Expr)>,
    },
    Insert {
        table_name: Ident,
    },
    Delete,
    Map {
        new_fields: Vec<(Ident, Type, Expr)>,
    },
    Unique {
        unique_field: Ident,
        from_field: Ident,
    },
    Filter(Expr),
    Row {
        fields: Vec<(Ident, Type, Expr)>,
    },
    Sort {
        fields: Vec<(Ident, SortOrder, Span)>,
    },
    Fold {
        initial: Vec<(Ident, Type, Expr)>,
        update: Vec<(Ident, Expr)>,
    },
    Assert(Expr),
    Collect,
}

#[derive(Debug)]
pub(super) struct Connector {
    /// single (~>) or stream (|>)
    pub single: bool,
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
    GenPK { field: Ident },
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
    pub cols: Vec<(Ident, syn::Type)>,
    pub cons: Vec<Constraint>,
}

#[derive(Debug)]
pub(super) struct Query {
    pub name: Ident,
    pub params: Vec<(Ident, syn::Type)>,
    pub streams: Vec<StreamExpr>,
}

#[derive(Debug)]
pub(super) struct BackendImpl {
    pub name: Ident,
    pub target: Ident,
}

#[derive(Debug)]
pub(super) struct Ast {
    pub backends: Vec<BackendImpl>,
    pub tables: Vec<Table>,
    pub queries: Vec<Query>,
}
