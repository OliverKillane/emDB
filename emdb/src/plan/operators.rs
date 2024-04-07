use super::{Data, Key, Plan, Query, ScalarType, Table, TableAccess};
use proc_macro2::Ident;
use std::collections::HashMap;
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

/// Apply a series of updates from a stream, the updated rows are propagated
/// INV: mapping and output have the same fields
/// INV: mapping expressions only contain fields from input and globals
/// INV: mapping assignment only contains fields from referenced table
pub struct Update {
    pub input: Key<DataFlow>,
    pub reference: Ident,
    pub table: Key<Table>,
    pub mapping: HashMap<Ident, Expr>,
    pub output: Key<DataFlow>,
}

/// Insert a single row or a stream into a table, the inserted rows
/// are propagated
/// INV: input and output have the same fields
/// INV: input has same fields as table
pub struct Insert {
    pub input: Key<DataFlow>,
    pub table: Key<Table>,
    pub out_ref: Ident,
    pub output: Key<DataFlow>,
}

/// Delete a single row or a stream from a table by reference,
/// the deleted rows are propagated
/// INV: input is a stream or single of row references
/// INV: output contains the tuple of removed values, same fields as table
pub struct Delete {
    pub input: Key<DataFlow>,
    pub reference: Ident,
    pub table: Key<Table>,
    pub output: Key<DataFlow>,
}

/// Gets a unique row from a table
pub struct GetUnique {
    pub input: Key<DataFlow>,
    pub access: TableAccess,
    pub from: Ident,
    pub table: Key<Table>,
    pub field: Ident,
    pub out: Ident,
    pub output: Key<DataFlow>,
}

/// Scan a table to generate a stream (optionally of references)
/// INV: if refs then output is a record of ref.
pub struct Scan {
    pub access: TableAccess,
    pub table: Key<Table>,
    pub output: Key<DataFlow>,
}

/// Dereference a table reference and place in a variable
/// INV: the 'named' not present in the input record
pub struct DeRef {
    pub input: Key<DataFlow>,
    pub reference: Ident,
    pub access: TableAccess,
    pub named: Ident,
    pub table: Key<Table>,
    pub output: Key<DataFlow>,
}

pub enum SortOrder {
    Asc,
    Desc,
}

pub struct FoldField {
    pub data_type: Key<ScalarType>,
    pub initial: Expr,
    pub update: Expr,
}

/// Applying a function over a stream of values
/// INV: output fields match mapping fields
/// INV: mapping expressions only contain fields from input and globals
pub struct Map {
    pub input: Key<DataFlow>,
    pub mapping: HashMap<Ident, Expr>,
    pub output: Key<DataFlow>,
}

/// A fold operation over a stream of values
/// INV: initial fields only contain globals
/// INV: update expressions only contain fields from input, initial and globals
/// INV: output matches initial types
pub struct Fold {
    pub input: Key<DataFlow>,
    pub fold_fields: HashMap<Ident, FoldField>,
    pub output: Key<DataFlow>,
}

/// Filter a stream of values
/// INV: predicate expression only contains fields from input and globals
pub struct Filter {
    pub input: Key<DataFlow>,
    pub predicate: Expr,
    pub output: Key<DataFlow>,
}

/// Sort the input given some keys and ordering
/// INV: input and output must have the same fields
/// INV: input and output must both be streams
/// INV: The identified fields must exist in the input
pub struct Sort {
    pub input: Key<DataFlow>,
    pub sort_order: Vec<(Ident, SortOrder)>,
    pub output: Key<DataFlow>,
}

/// Assert a boolean expression over a stream, or single value
/// INV: input type is same as output type
/// INV: predicate expression only contains fields from input and globals
pub struct Assert {
    pub input: Key<DataFlow>,
    pub assert: Expr,
    pub output: Key<DataFlow>,
}

/// A fold that outputs a collection of all data input, included here to allow
/// the optimiser to reason more easily about the data structure size & type
/// given many queries collect multiple rows.
/// - Generates the [ScalarType::Bag] types
/// - Could technically be implemented as a fold, but more difficult to reason
///   about folds generally, and we want the backend to determine the bag's type
pub struct Collect {
    pub input: Key<DataFlow>,
    pub into: Ident,
    pub output: Key<DataFlow>,
}

/// Take the top n from a stream, discarding the rest
pub struct Take {
    pub input: Key<DataFlow>,
    pub top_n: Expr,
    pub output: Key<DataFlow>,
}

/// An n-way join of either equijoin, predicate join, > join, or cross
pub struct Join {
    // TODO
}

/// group by a field and aggregate the results
pub struct GroupBy {
    pub input: Key<DataFlow>,
    pub group_on: Ident,
    pub aggregate_start: Key<DataFlow>,
    pub aggregate_end: Key<DataFlow>,
    pub output: Key<DataFlow>,
}

/// Given an operator output, multiply it into multiple outputs
pub struct Fork {
    pub input: Key<DataFlow>,
    pub outputs: Vec<Key<DataFlow>>,
}

/// Merge a number of streams into one
pub struct Union {
    pub inputs: Vec<Key<DataFlow>>,
    pub output: Key<DataFlow>,
}

/// Generate a single row
/// INV: output matches fields
/// INV: output is a single
pub struct Row {
    pub fields: HashMap<Ident, Expr>,
    pub output: Key<DataFlow>,
}

/// Return values from a query
pub struct Return {
    pub input: Key<DataFlow>,
}

/// Stream in and then discard
pub struct Discard {
    pub input: Key<DataFlow>,
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pub modify_operator_enum)]
pub enum ModifyOperator {
    Update,
    Insert,
    Delete,
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pub access_operator_enum)]
pub enum AccessOperator {
    GetUnique,
    Scan,
    DeRef,
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pure_operator_enum)]
pub enum PureOperator {
    Map,
    Fold,
    Filter,
    Sort,
    Assert,
    Collect,
    Take,
    Join,
    GroupBy,
    Fork,
    Union,
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(flow_operator_enum)]
pub enum FlowOperator {
    Row,
    Return,
    Discard,
}

pub struct Operator {
    pub query: Key<Query>,
    pub kind: OperatorKind,
}

#[enumtrait::quick_from]
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

    pub fn get_dataflow(&self, k: Key<DataFlow>) -> &DataFlow {
        self.dataflow.get(k).unwrap()
    }

    pub fn get_operator(&self, k: Key<Operator>) -> &Operator {
        self.operators.get(k).unwrap()
    }
}
