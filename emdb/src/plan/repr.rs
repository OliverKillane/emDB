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
    pub(crate) read: bool,
    pub(crate) write: bool,
    pub(crate) unique: UniqueCons,
}

pub(crate) struct LogicalRowConstraint {
    pub(crate) insert: bool,
    pub(crate) delete: bool,
    pub(crate) limit: Option<(Expr, Option<Ident>)>,
    pub(crate) genpk: Option<(Ident, Option<Ident>)>,
    pub(crate) preds: Vec<(Expr, Option<Ident>)>,
}

#[derive(Clone)]
pub(crate) enum ColumnType {
    Concrete(Type),
    Synthetic {
        implemented: Option<Type>,
        reference: Expr,
    },
}

pub(crate) struct LogicalColumn {
    pub(crate) constraints: LogicalColumnConstraint,
    pub(crate) data_type: ColumnType,
}

/// Add synthetic rows
pub(crate) struct LogicalTable {
    pub(crate) name: Ident,
    pub(crate) constraints: LogicalRowConstraint,
    pub(crate) columns: HashMap<Ident, LogicalColumn>,
}

impl LogicalTable {
    pub(crate) fn get_type(&self) -> Record {
        let mut fields = HashMap::new();
        for (name, col) in &self.columns {
            // TODO unchecked insert
            fields.insert(name.clone(), RecordData::Rust(col.data_type.clone()));
        }
        Record {
            fields,
            stream: true,
        }
    }
}

#[derive(Clone)]
pub(crate) enum RecordData {
    Record(Record),
    Ref(GenIndex<LogicalTable>), // Represented by Ref<table type>
    Rust(Type),
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

// TODO: Each operator is marked by the query it is from
type EdgeKey = Option<GenIndex<Edge>>;
pub(crate) enum LogicalOperator {
    // Table Access ============================================================
    /// Apply a series of updates from a stream, the updated rows are propagated
    /// INV: mapping and output have the same fields
    /// INV: mapping expressions only contain fields from input and globals
    /// INV: mapping assignment only contains fields from referenced table
    Update {
        input: EdgeKey,
        reference: Expr, // todo fix
        table: GenIndex<LogicalTable>,
        mapping: HashMap<Ident, (Type, Expr)>,
        output: EdgeKey,
    },

    /// Insert a single row or a stream into a table, the inserted rows
    /// are propagated
    /// INV: input and output have the same fields
    /// INV: input has same fields as table
    Insert {
        input: EdgeKey,
        table: GenIndex<LogicalTable>,
        output: EdgeKey,
    },

    /// Delete a single row or a stream from a table by reference,
    /// the deleted rows are propagated
    /// INV: input is a stream or single of row references
    /// INV: output contains the tuple of removed values, same fields as table
    Delete {
        input: EdgeKey,
        table: GenIndex<LogicalTable>,
        output: EdgeKey,
    },

    /// Gets a unique row from a table
    /// INV: the input_val contains a single value of the type of the unique
    ///      field in the table
    Unique {
        unique_field: Ident,
        refs: bool,
        from_expr: Expr,
        table: GenIndex<LogicalTable>,
        output: EdgeKey,
    },

    /// Scan a table to generate a stream (optionally of references)
    /// INV: if refs then output is a record of ref.
    Scan {
        refs: bool,
        table: GenIndex<LogicalTable>,
        output: EdgeKey,
    },

    // Basic Operations ========================================================
    /// Applying a function over a stream of values
    /// INV: output fields match mapping fields
    /// INV: mapping expressions only contain fields from input and globals
    Map {
        input: EdgeKey,
        mapping: HashMap<Ident, (Type, Expr)>,
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
        fields: HashMap<Ident, (Type, Expr)>,
        output: EdgeKey,
    },

    /// Given an operator output, multiply it into multiple outputs
    Multiply {
        input: EdgeKey,
        outputs: HashSet<EdgeKey>,
    },

    // Query Wrangling =========================================================
    /// A parameter from a query

    /// The end of a stream (may be referenced by a return, or discarded)
    End { input: EdgeKey },
}

impl LogicalOperator {
    fn get_only_output(&self) -> Option<EdgeKey> {
        match self {
            LogicalOperator::Update { output, .. }
            | LogicalOperator::Insert { output, .. }
            | LogicalOperator::Delete { output, .. }
            | LogicalOperator::Unique { output, .. }
            | LogicalOperator::Scan { output, .. }
            | LogicalOperator::Map { output, .. }
            | LogicalOperator::Fold { output, .. }
            | LogicalOperator::Filter { output, .. }
            | LogicalOperator::Sort { output, .. }
            | LogicalOperator::Assert { output, .. }
            | LogicalOperator::Row { output, .. } => Some(*output),
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
    returnval: GenIndex<LogicalOperator>,
}

pub(crate) struct LogicalQueryParams {
    pub(crate) name: Ident,
    pub(crate) data_type: Type,
}

pub(crate) struct LogicalPlan {
    pub(crate) queries: GenArena<LogicalQuery>,
    pub(crate) tables: GenArena<LogicalTable>,
    pub(crate) record_types: GenArena<Record>,
    pub(crate) operators: GenArena<LogicalOperator>,
    pub(crate) operator_edges: GenArena<Edge>,
}

impl LogicalPlan {
    pub fn new() -> Self {
        LogicalPlan {
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
