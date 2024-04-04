use std::collections::HashMap;

use super::{Data, Key, Query, ScalarType, Table, TableAccess, Plan};
use proc_macro2::Ident;
use syn::Expr;

pub enum DataFlow {
    Conn {
        from: Key<Operator>,
        to: Key<Operator>,
        with: Data,
    },

    /// (For graph construction)
    /// INV: None exist in the edges after graph construction
    Incomplete { from: Key<Operator>, with: Data },

    /// (For graph construction)
    /// INV: None exist in the edges after graph construction
    Null,
}

pub enum ModifyOperator {
    /// Apply a series of updates from a stream, the updated rows are propagated
    /// INV: mapping and output have the same fields
    /// INV: mapping expressions only contain fields from input and globals
    /// INV: mapping assignment only contains fields from referenced table
    Update {
        input: Key<DataFlow>,
        reference: Ident,
        table: Key<Table>,
        mapping: HashMap<Ident, Expr>,
        output: Key<DataFlow>,
    },

    /// Insert a single row or a stream into a table, the inserted rows
    /// are propagated
    /// INV: input and output have the same fields
    /// INV: input has same fields as table
    Insert {
        input: Key<DataFlow>,
        table: Key<Table>,
        out_ref: Ident,
        output: Key<DataFlow>,
    },

    /// Delete a single row or a stream from a table by reference,
    /// the deleted rows are propagated
    /// INV: input is a stream or single of row references
    /// INV: output contains the tuple of removed values, same fields as table
    Delete {
        input: Key<DataFlow>,
        reference: Ident,
        table: Key<Table>,
        output: Key<DataFlow>,
    },
}

pub enum AccessOperator {
    /// Gets a unique row from a table
    Unique {
        input: Key<DataFlow>,
        access: TableAccess,
        from: Ident,
        table: Key<Table>,
        field: Ident,
        out: Ident,
        output: Key<DataFlow>,
    },

    /// Scan a table to generate a stream (optionally of references)
    /// INV: if refs then output is a record of ref.
    Scan {
        access: TableAccess,
        table: Key<Table>,
        output: Key<DataFlow>,
    },

    /// Dereference a table reference and place in a variable
    /// INV: the 'named' not present in the input record
    DeRef {
        input: Key<DataFlow>,
        reference: Ident,
        access: TableAccess,
        named: Ident,
        table: Key<Table>,
        output: Key<DataFlow>,
    },
}

pub enum SortOrder {
    Asc,
    Desc,
}

pub struct FoldField {
    pub data_type: Key<ScalarType>,
    pub initial: Expr,
    pub update: Expr
}

pub enum PureOperator {
    /// Applying a function over a stream of values
    /// INV: output fields match mapping fields
    /// INV: mapping expressions only contain fields from input and globals
    Map {
        input: Key<DataFlow>,
        mapping: HashMap<Ident, Expr>,
        output: Key<DataFlow>,
    },

    /// A fold operation over a stream of values
    /// INV: initial fields only contain globals
    /// INV: update expressions only contain fields from input, initial and globals
    /// INV: output matches initial types
    Fold {
        input: Key<DataFlow>,
        fold_fields: HashMap<Ident, FoldField>,
        output: Key<DataFlow>,
    },

    /// Filter a stream of values
    /// INV: predicate expression only contains fields from input and globals
    Filter {
        input: Key<DataFlow>,
        predicate: Expr,
        output: Key<DataFlow>,
    },

    /// Sort the input given some keys and ordering
    /// INV: input and output must have the same fields
    /// INV: input and output must both be streams
    /// INV: The identified fields must exist in the input
    Sort {
        input: Key<DataFlow>,
        sort_order: Vec<(Ident, SortOrder)>,
        output: Key<DataFlow>,
    },

    /// Assert a boolean expression over a stream, or single value
    /// INV: input type is same as output type
    /// INV: predicate expression only contains fields from input and globals
    Assert {
        input: Key<DataFlow>,
        assert: Expr,
        output: Key<DataFlow>,
    },

    /// A fold that outputs a collection of all data input, included here to allow
    /// the optimiser to reason more easily about the data structure size & type
    /// given many queries collect multiple rows.
    /// - Generates the [ScalarType::Bag] types
    /// - Could technically be implemented as a fold, but more difficult to reason
    ///   about folds generally, and we want the backend to determine the bag's type
    Collect {
        input: Key<DataFlow>,
        into: Ident,
        output: Key<DataFlow>,
    },

    /// Take the top n from a stream, discarding the rest
    Take {
        input: Key<DataFlow>,
        top_n: Expr,
        output: Key<DataFlow>,
    },

    /// An n-way join of either equijoin, predicate join, > join, or cross 
    Join {
        // todo
    },

    /// group by a field and aggregate the results
    GroupBy {
        input: Key<DataFlow>,
        group_on: Ident,
        aggregate_start: Key<DataFlow>,
        aggregate_end: Key<DataFlow>, 
        output: Key<DataFlow>,
    },

    /// Given an operator output, multiply it into multiple outputs
    Fork {
        input: Key<DataFlow>,
        outputs: Vec<Key<DataFlow>>,
    },

    /// Merge a number of streams into one
    Union {
        inputs: Vec<Key<DataFlow>>,
        output: Key<DataFlow>,
    },
}

pub enum FlowOperator {
    /// Generate a single row
    /// INV: output matches fields
    /// INV: output is a single
    Row {
        fields: HashMap<Ident, Expr>,
        output: Key<DataFlow>,
    },

    /// Return values from a query
    Return { input: Key<DataFlow> },

    /// Stream in and then discard
    Discard { input: Key<DataFlow> },
}

pub struct Operator {
    pub query: Key<Query>,
    pub kind: OperatorKind,
}

pub enum OperatorKind {
    /// Used for operators that modify table state.
    /// Within a query they need to be strictly ordered (as one may read the modifications from another)
    /// INV: [`OperatorKind::Modify.modify_after`] is a [`OperatorKind::Modify`] or [`OperatorKind::Access`]
    Modify {
        modify_after: Option<Key<Operator>>,
        op: ModifyOperator,
    },

    /// INV: `access_after` is a [`OperatorKind::Modify`] or [`OperatorKind::Access`]
    Access {
        access_after: Option<Key<Operator>>,
        op: AccessOperator,
    },

    Pure(PureOperator),
    Flow(FlowOperator),
}

impl Plan {
    pub fn get_mut_dataflow(&mut self, k: Key<DataFlow>) -> &mut DataFlow {
        self.dataflow.get_mut(k).unwrap()
    } 
}