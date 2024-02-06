use std::collections::{HashMap, HashSet};

use proc_macro2::Ident;
use syn::{Expr, Type};
use typed_generational_arena::{Arena as GenArena, Index as GenIndex};

pub(crate) type ComplexCons<T> = Option<(T, Option<Ident>)>;

pub(crate) enum UniqueCons {
    Unique(Option<Ident>),
    NotUnique,
}

pub(crate) struct LogicalColumnConstraint {
    pub(crate) read: bool,
    pub(crate) write: bool,
    pub(crate) unique: UniqueCons,
}

pub(crate) struct LogicalRowConstraint {
    pub(crate) insert: bool,
    pub(crate) delete: bool,
    pub(crate) limit: ComplexCons<Expr>,
    pub(crate) genpk: ComplexCons<Ident>,
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

pub(crate) enum RecordData {
    Record(Record),
    Ref(GenIndex<LogicalTable>), // Represented by Ref<table type>
    Rust(Type),
}
pub(crate) struct Record {
    pub(crate) fields: HashMap<Ident, RecordData>,
}

pub(crate) enum Edge {
    Bi {
        from: GenIndex<Operator>,
        to: GenIndex<Operator>,
        with: Record,
        stream: bool,
    },
    Uni {
        from: GenIndex<Operator>,
        with: Record,
        stream: bool,
    },

    /// Used for incomplete graphs during construction
    Null,
}

pub(crate) enum Operator {
    // Table Access ============================================================
    /// Apply a series of updates from a stream, the updated rows are propagated
    /// INV: mapping and output have the same fields
    /// INV: mapping expressions only contain fields from input and globals
    /// INV: mapping assignment only contains fields from referenced table
    Update {
        input: GenIndex<Edge>,
        reference: Expr, // todo fix
        table: GenIndex<LogicalTable>,
        mapping: HashMap<Ident, (Type, Expr)>,
        output: GenIndex<Edge>,
    },

    /// Insert a single row or a stream into a table, the inserted rows
    /// are propagated
    /// INV: input and output have the same fields
    /// INV: input has same fields as table
    Insert {
        input: GenIndex<Edge>,
        table: GenIndex<LogicalTable>,
        output: GenIndex<Edge>,
    },

    /// Delete a single row or a stream from a table by reference,
    /// the deleted rows are propagated
    /// INV: input is a stream or single of row references
    /// INV: output contains the tuple of removed values, same fields as table
    Delete {
        input: GenIndex<Edge>,
        table: GenIndex<LogicalTable>,
        output: GenIndex<Edge>,
    },

    /// Gets a unique row from a table
    Unique {
        unique_field: Ident,
        refs: bool,
        table: GenIndex<LogicalTable>,
        output: GenIndex<Edge>,
    },

    /// Scan a table to generate a stream (optionally of references)
    /// INV: if refs then output is a record of ref.
    Scan {
        refs: bool,
        table: GenIndex<LogicalTable>,
        output: GenIndex<Edge>,
    },

    // Basic Operations ========================================================
    /// Applying a function over a stream of values
    /// INV: output fields match mapping fields
    /// INV: mapping expressions only contain fields from input and globals
    Map {
        input: GenIndex<Edge>,
        mapping: HashMap<Ident, (Type, Expr)>,
        output: GenIndex<Edge>,
    },

    /// A fold operation over a stream of values
    /// INV: initial fields only contain globals
    /// INV: update expressions only contain fields from input, initial and globals
    /// INV: output matches initial types
    Fold {
        input: GenIndex<Edge>,
        initial: HashMap<Ident, (Type, Expr)>,
        update: HashMap<Ident, Expr>,
        output: GenIndex<Edge>,
    },

    /// Filter a stream of values
    /// INV: predicate expression only contains fields from input and globals
    Filter {
        input: GenIndex<Edge>,
        predicate: Expr,
        output: GenIndex<Edge>,
    },

    /// Sort the input given some keys and ordering
    /// INV: input and output must have the same fields
    /// INV: input and output must both be streams
    /// INV: The identified fields must exist in the input
    Sort {
        input: GenIndex<Edge>,
        sort_order: Vec<(Ident, SortOrder)>,
        output: GenIndex<Edge>,
    },

    /// Assert a boolean expression over a stream, or single value
    /// INV: input type is same as output type
    /// INV: predicate expression only contains fields from input and globals
    Assert {
        input: GenIndex<Edge>,
        assert: Expr,
        output: GenIndex<Edge>,
    },

    // Stream Creation =========================================================
    /// Generate a single row
    /// INV: output matches fields
    /// INV: output is a single
    Row {
        fields: HashMap<Ident, (Type, Expr)>,
        output: GenIndex<Edge>,
    },

    /// Given an operator output, multiply it into multiple outputs
    Multiply {
        input: GenIndex<Edge>,
        outputs: HashSet<GenIndex<Edge>>,
    },

    // Query Wrangling =========================================================
    /// A parameter from a query
    /// INV: output is a single, with the same name and type as parameter
    QueryParam {
        param: GenIndex<LogicalQueryParams>,
        output: GenIndex<Edge>,
    },

    /// The end of a stream (may be referenced by a return, or discarded)
    End { input: GenIndex<Edge> },
}

impl Operator {
    fn get_only_output(&self) -> Option<GenIndex<Edge>> {
        match self {
            Operator::Update { output, .. }
            | Operator::Insert { output, .. }
            | Operator::Delete { output, .. }
            | Operator::Unique { output, .. }
            | Operator::Scan { output, .. }
            | Operator::Map { output, .. }
            | Operator::Fold { output, .. }
            | Operator::Filter { output, .. }
            | Operator::Sort { output, .. }
            | Operator::Assert { output, .. }
            | Operator::Row { output, .. }
            | Operator::QueryParam { output, .. } => Some(*output),
            _ => None,
        }
    }
}

pub(crate) enum SortOrder {
    Asc,
    Desc,
}

pub(crate) struct LogicalQuery {
    name: Ident,
    params: Vec<LogicalQueryParams>,
    /// INV is an [Operator::End] operator
    returnval: GenIndex<Operator>,
}

pub(crate) struct LogicalQueryParams {
    pub(crate) name: Ident,
    pub(crate) data_type: Type,
}

pub(crate) struct LogicalPlan {
    pub(crate) queryparams: GenArena<LogicalQueryParams>,
    pub(crate) queries: GenArena<LogicalQuery>,
    pub(crate) tables: GenArena<LogicalTable>,
    pub(crate) record_types: GenArena<Record>,
    pub(crate) operators: GenArena<Operator>,
    pub(crate) operator_edges: GenArena<Edge>,
}

impl LogicalPlan {
    pub fn new() -> Self {
        LogicalPlan {
            queryparams: GenArena::new(),
            queries: GenArena::new(),
            tables: GenArena::new(),
            record_types: GenArena::new(),
            operators: GenArena::new(),
            operator_edges: GenArena::new(),
        }
    }

    fn validate(&self) {}

    fn replace_operator() {}
}
