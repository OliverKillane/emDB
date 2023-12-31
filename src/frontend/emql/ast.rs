// TODO: parameterize by span type (want to go to resolved AST spanned by types)
use proc_macro2::{Ident, Span};
use syn::{self, Expr};

pub(super) enum Operator {
    Ret { ret_span: Span },
    Ref { ref_span: Span, table_name: Ident },
    Let { let_span: Span, var_name: Ident },
    Use { use_span: Span, var_name: Ident },
    FuncOp { fn_span: Span, op: FuncOp },
}

// TODO: Joins, GroupBy
pub(super) enum FuncOp {
    Scan {
        table_name: Ident,
    },
    Update {
        fields: Vec<(Ident, Expr)>,
    },
    Insert {
        table_name: Ident,
    },
    Delete,
    Map {
        new_fields: Vec<(Ident, Expr)>,
    },
    Unique {
        table: Ident,
        unique_field: Ident,
        from_field: Ident,
    },
    Filter(Expr),
    Row {
        fields: Vec<(Ident, Expr)>,
    },
    Sort {
        fields: Vec<(Ident, Ident)>,
    },
    Fold {
        initial: Expr,
        op: Expr,
    },
    Assert(Expr),
}

pub(super) struct Connector {
    /// single (~>) or stream (|>)
    pub single: bool,
    pub span: Span,
}

pub(super) struct StreamExpr {
    pub op: Operator,
    pub con: Option<(Connector, Box<StreamExpr>)>,
}

pub(super) enum ConstraintExpr {
    Unique { field: Ident },
    Pred(Expr),
    GenPK { field: Ident },
    Limit { size: Expr },
}

pub(super) struct Constraint {
    pub alias: Option<Ident>,
    pub method_span: Span,
    pub expr: ConstraintExpr,
}

pub(super) struct Table {
    pub name: Ident,
    pub cols: Vec<(Ident, syn::Type)>,
    pub cons: Vec<Constraint>,
}

pub(super) struct Query {
    pub name: Ident,
    pub params: Vec<(Ident, syn::Type)>,
    pub streams: Vec<StreamExpr>,
}

pub(super) struct AST {
    pub name: Ident,
    pub tables: Vec<Table>,
    pub queries: Vec<Query>,
}
