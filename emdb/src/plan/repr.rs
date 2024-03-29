use std::collections::{HashMap, HashSet};

use proc_macro2::Ident;
use syn::{Expr, Type};
use typed_generational_arena::{Index, NonzeroGeneration, StandardArena as GenArena};

pub(crate) type GenIndex<T> = Index<T, usize, NonzeroGeneration<usize>>;

pub(crate) enum UniqueCons {
    Unique(Option<Ident>),
    NotUnique,
}

pub(crate) struct LogicalColumnConstraint {
    pub(crate) unique: UniqueCons,
}

pub(crate) struct LogicalRowConstraint {
    pub(crate) limit: Option<(Expr, Option<Ident>)>,
    pub(crate) preds: Vec<(Expr, Option<Ident>)>,
}

pub(crate) struct LogicalColumn {
    pub(crate) constraints: LogicalColumnConstraint,
    pub(crate) data_type: Type,
}

pub(crate) struct LogicalTable {
    pub(crate) name: Ident,
    pub(crate) constraints: LogicalRowConstraint,
    pub(crate) columns: HashMap<Ident, LogicalColumn>,
}


#[derive(Clone)]
pub(crate) enum ScalarType {
    Ref(TableKey), // Represented by Ref<table type>
    Rust(Type),
    Bag(Record), // Used by collect
}

#[derive(Clone)]
pub(crate) enum RecordData {
    Record(Record),
    Scalar(ScalarType),
}
#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) fields: HashMap<Ident, RecordData>,
    pub(crate) stream: bool,
}

pub(crate) enum Edge {
    Bi {
        from: GenIndex<LogicalOperator>,
        to: GenIndex<LogicalOperator>,
        with: Record,
    },
    Uni {
        from: GenIndex<LogicalOperator>,
        with: Record,
    },

    /// Used for incomplete graphs during construction
    Null,
}

pub(crate) type EdgeKey = GenIndex<Edge>;

pub(crate) enum TableAccess {
    Ref,
    AllCols,
    Selection(Vec<Ident>),
}

pub(crate) type QueryKey = GenIndex<LogicalQuery>;

pub(crate) struct LogicalOperator {
    pub query: Option<QueryKey>,
    pub operator: LogicalOp,
}

pub(crate) type TableKey = GenIndex<LogicalTable>;

pub(crate) enum LogicalOp {
    // Table Access ============================================================
    /// Apply a series of updates from a stream, the updated rows are propagated
    /// INV: mapping and output have the same fields
    /// INV: mapping expressions only contain fields from input and globals
    /// INV: mapping assignment only contains fields from referenced table
    Update {
        input: EdgeKey,
        reference: Ident,
        table: TableKey,
        mapping: HashMap<Ident, Expr>,
        output: EdgeKey,
    },

    /// Insert a single row or a stream into a table, the inserted rows
    /// are propagated
    /// INV: input and output have the same fields
    /// INV: input has same fields as table
    Insert {
        input: EdgeKey,
        table: TableKey,
        output: EdgeKey,
    },

    /// Delete a single row or a stream from a table by reference,
    /// the deleted rows are propagated
    /// INV: input is a stream or single of row references
    /// INV: output contains the tuple of removed values, same fields as table
    Delete {
        input: EdgeKey,
        reference: Ident,
        table: TableKey,
        output: EdgeKey,
    },

    /// Gets a unique row from a table
    /// INV: the input_val contains a single value of the type of the unique
    ///      field in the table
    Unique {
        unique_field: Ident,
        access: TableAccess,
        from_expr: Expr,
        table: TableKey,
        output: EdgeKey,
    },

    /// Scan a table to generate a stream (optionally of references)
    /// INV: if refs then output is a record of ref.
    Scan {
        access: TableAccess,
        table: TableKey,
        output: EdgeKey,
    },

    /// Dereference a table reference and place in a variable
    /// INV: the 'named' not present in the input record
    DeRef {
        input: EdgeKey,
        reference: Ident,
        named: Ident,
        table: TableKey,
        output: EdgeKey,
    },

    // Basic Operations ========================================================
    /// Applying a function over a stream of values
    /// INV: output fields match mapping fields
    /// INV: mapping expressions only contain fields from input and globals
    Map {
        input: EdgeKey,
        mapping: HashMap<Ident, Expr>,
        output: EdgeKey,
    },

    /// A fold operation over a stream of values
    /// INV: initial fields only contain globals
    /// INV: update expressions only contain fields from input, initial and globals
    /// INV: output matches initial types
    Fold {
        input: EdgeKey,
        initial: HashMap<Ident, (Type, Expr)>,
        update: HashMap<Ident, Expr>,
        output: EdgeKey,
    },

    /// Filter a stream of values
    /// INV: predicate expression only contains fields from input and globals
    Filter {
        input: EdgeKey,
        predicate: Expr,
        output: EdgeKey,
    },

    /// Sort the input given some keys and ordering
    /// INV: input and output must have the same fields
    /// INV: input and output must both be streams
    /// INV: The identified fields must exist in the input
    Sort {
        input: EdgeKey,
        sort_order: Vec<(Ident, SortOrder)>,
        output: EdgeKey,
    },

    /// Assert a boolean expression over a stream, or single value
    /// INV: input type is same as output type
    /// INV: predicate expression only contains fields from input and globals
    Assert {
        input: EdgeKey,
        assert: Expr,
        output: EdgeKey,
    },

    /// Take the union of several streams
    /// INV: the inputs are all the same type, and are all streams
    /// INV: the output is the same type as inputs
    Union {
        inputs: HashSet<EdgeKey>,
        output: EdgeKey,
    },

    // Stream Creation =========================================================
    /// Generate a single row
    /// INV: output matches fields
    /// INV: output is a single
    Row {
        fields: HashMap<Ident, Expr>,
        output: EdgeKey,
    },

    /// Given an operator output, multiply it into multiple outputs
    Multiply {
        input: EdgeKey,
        outputs: HashSet<EdgeKey>,
    },

    // Stream Control ==========================================================
    /// Return values from a query
    Return { input: EdgeKey },


    // Logical Sugar ===========================================================
    /// A fold that outputs a collection of all data input, included here to allow 
    /// the optimiser to reason more easily about the data structure size & type
    /// given many queries collect multiple rows.
    Collect {input: EdgeKey, output: EdgeKey}
}

pub(crate) enum SortOrder {
    Asc,
    Desc,
}

pub(crate) type OpKey = GenIndex<LogicalOperator>;

pub(crate) struct LogicalQuery {
    pub name: Ident,
    pub params: Vec<LogicalQueryParams>,
    /// INV is a [LogicalOp::Return]
    pub returnval: Option<OpKey>,
}

pub(crate) struct LogicalQueryParams {
    pub(crate) name: Ident,
    pub(crate) data_type: ScalarType,
}

pub(crate) struct LogicalPlan {
    pub(crate) queries: GenArena<LogicalQuery>,
    pub(crate) tables: GenArena<LogicalTable>,
    pub(crate) operators: GenArena<LogicalOperator>,
    pub(crate) operator_edges: GenArena<Edge>,
}

impl LogicalPlan {
    pub fn new() -> Self {
        LogicalPlan {
            queries: GenArena::new(),
            tables: GenArena::new(),
            operators: GenArena::new(),
            operator_edges: GenArena::new(),
        }
    }
}
