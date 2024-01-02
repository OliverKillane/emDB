//! The logical plan data structure for emDB
//!
//! ## Purpose
//! All logical optimisations occur on this plan.
//! - Frontend & Backend agnostic
//! - Contains both rust and emDB types.
//! [TODO] Add printout
use std::collections::{HashMap, HashSet};
use syn;
use typed_arena::Arena;
use typed_generational_arena::{Arena as GenArena, Index as GenIndex};

pub(crate) enum SingleType {
    /// A Rust Type
    RsType(syn::Type),

    /// Immutable string representation
    Text { length: usize },
}

pub(crate) enum DataType {
    /// A series of data, streamed through operators.
    Stream(Box<DataType>),

    /// A normal database row, propagated through operators
    Row(RowType),

    /// It is a reference, for use in ref streams (e.g inserts to a table).
    /// - Not necessarily just a rust reference (e.g a LockGuard, borrow of some
    ///   string type reference)
    /// - Can reference a table, or an intermediate materialised table.
    Ref(Box<DataType>),

    /// A Rust Type
    Single(SingleType),
}

/// ## Contains the entire logical plan.
pub(crate) struct LogicalPlan<'a> {
    stream_operators: GenArena<StreamNode<'a>>,
    single_operators: GenArena<SingleNode<'a>>,
    var_single: Arena<VarSingle>,
    methods: Arena<CallOperator<'a>>,
    tables: GenArena<Table>,
}

pub(crate) struct VarSingle {
    datatype: DataType,
}

/// The return value of a Call Operator
/// - Some stream returns can be optimised into references, or lock guards.
/// - Some need to collect to a single.
pub(crate) enum ReturnVal<'a> {
    None,
    Single(SingleNode<'a>),
    Stream(StreamNode<'a>),
}

pub(crate) struct CallOperator<'a> {
    name: String,
    args: HashMap<String, &'a VarSingle>,
    returnval: ReturnVal<'a>,
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum RowOrdering {
    Ascending,
    Descending,
}

pub(crate) type ColIndex = usize;

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum Constraint {
    Unique { col: ColIndex },
    Ordered { ords: Vec<(ColIndex, RowOrdering)> },
    Asserts { col: ColIndex },
    LimitRows { size: usize },
    References { col: TableIndex },
}

pub(crate) type RowType = Vec<SingleType>;
pub(crate) struct Table {
    cols: RowType,
    initialdata: Vec<Vec<syn::Expr>>,
    constraints: HashSet<Constraint>,
}

pub(crate) type SingleIndex<'a> = GenIndex<SingleNode<'a>>;
pub(crate) type StreamIndex<'a> = GenIndex<StreamNode<'a>>;
pub(crate) type TableIndex = GenIndex<Table>;

pub(crate) struct StreamNode<'a> {
    op: StreamOperator<'a>,
    output: DataType,
}

pub(crate) enum StreamOperator<'a> {
    Scan {
        source: TableIndex,
    },
    Map {
        prev: StreamIndex<'a>,
        func: (),
    },
    Sort {
        prev: StreamIndex<'a>,
        col: (),
        sort: (),
    },
    Join {
        left: StreamIndex<'a>,
        right: StreamIndex<'a>,
        cond: (),
    },
    Limit {
        prev: StreamIndex<'a>,
        vol: SingleIndex<'a>,
    },
    Filter {
        prev: StreamIndex<'a>,
        cond: (),
    },

    /// Allows users to repeat a stream a given number of times
    /// - `1,2,3` through repeat(2) becomes `1,1,2,2,3,3`
    Repeat {
        prev: StreamIndex<'a>,
        times: SingleIndex<'a>,
    },

    /// Groups by a given column identifier from a row stream
    /// - breaks the pipeline breaker (partially)
    GroupBy {
        prev: StreamIndex<'a>,
        col: ColIndex,
    },

    /// Appends two streams of the same type
    Union {
        left: StreamIndex<'a>,
        right: StreamIndex<'a>,
    },

    Minus {
        left: StreamIndex<'a>,
        right: StreamIndex<'a>,
    },

    /// Converts a stream of values into a temporary table from which references
    /// can be streamed
    Materialise {
        prev: SingleIndex<'a>,
    },
}

pub(crate) struct SingleNode<'a> {
    op: SingleOperator<'a>,
    output: DataType,
}

pub(crate) enum SingleOperator<'a> {
    Unique {
        prev: StreamIndex<'a>,
        col: (),
        select: (),
    },
    Select {
        prev: StreamIndex<'a>,
        col: (),
        func: (),
    },
    Var {
        prev: &'a VarSingle,
    },
    Const {
        datatype: SingleType,
        value: syn::Expr,
    },
    /// Allows the user to constrain types
    Fold {
        prev: StreamIndex<'a>,
        collect_fun: (),
    },
    Merge {},
}
