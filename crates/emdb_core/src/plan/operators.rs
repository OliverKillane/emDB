//! # EMDB Logical Operators
//! ## Design
//! A small set of minimal operators, with as simple as plan as possible.
//! - Complete flexibility in operators (can have `n` connections, each associated
//!   with different data, as members of the operator)
//! - Arena based (allows [analyses](crate::analysis) to index nodes in the
//!   plan, without requiring additions to it)
//!
//! ## Shortcomings
//! For the side effecting operators, we want to move some attributes from the
//! stream, but propagate others.
//!
//! Ideally we would have something like:
//! ```text
//! Rec { a, b } -> insert(only take a) -> Rec { ref, b }
//! ```
//!
//! We could generalise this as a `remap` operator
//! ```text
//! Rec { a, b } -> remap( Rec{a} -> insert() -> return, Rec{b} -> return ) -> Rec { b })
//! ```
//!
//! In the [super] documentation the shortcomings note provides some of a suggestion:
//! - Could nest a context, but the operator type is restricted to ones that dont affect
//!   cardinality of output.

use super::{Context, Data, Key, Plan, RecordField, RecordType, Table};
use std::collections::HashMap;
use syn::Expr;

/// A complete data flow connection (only type allowed for valid, constructed plans)
pub struct DataFlowConn {
    pub from: Key<Operator>,
    pub to: Key<Operator>,
    pub with: Data,
}

/// Represents a connection to transfer data between two operators
pub enum DataFlow {
    Conn(DataFlowConn),

    /// (For graph construction)
    /// - `INV`: None exist in the edges after graph construction
    Incomplete {
        from: Key<Operator>,
        with: Data,
    },

    /// (For graph construction)
    /// - `INV`: None exist in the edges after graph construction
    Null,
}

impl DataFlow {
    pub fn get_conn(&self) -> &DataFlowConn {
        match self {
            DataFlow::Conn(c) => c,
            _ => panic!("Attempted to get connection from non-connection dataflow"),
        }
    }
}

/// Write to columns of a row in the table from a stream/single, specifying:
/// - The values to provide for columns (using references to the input)
/// - The field that contains the table reference to use.
///
/// Returns the input type, table is mutated.
/// ```text
/// RECORD -> update(use RECORD.{ .. } and Fields = |&RECORD| { .. } ) -> RECORD
/// ```
pub struct Update {
    pub input: Key<DataFlow>,

    /// `INV`: `reference` field is in the input
    pub reference: RecordField,

    pub table: Key<Table>,

    // `INV`: each field in mapping is in the table.
    pub mapping: HashMap<RecordField, Expr>,

    /// `INV`: All fields in the record are fields in the table
    pub update_type: Key<RecordType>,

    // `INV`: `input.with == output.with`
    pub output: Key<DataFlow>,
}

/// Insert the record as a field stream/single, producing a stream/single of row
/// references.
/// - return stream is of references to the inserted values.
///
/// ```text
/// TABLE::INSERT -> insert(TABLE) -> TABLE::REF
/// ```
pub struct Insert {
    pub input: Key<DataFlow>,
    pub table: Key<Table>,

    /// The single field to place the out_ref in.
    /// `INV`: `out_ref` is the only field in `output.with`
    pub out_ref: RecordField,

    pub output: Key<DataFlow>,
}

/// Delete a rows from a table using a row reference.
///
/// ```text
/// RECORD -> delete(Ruse RECORD.{ .. } ) -> RECORD
/// ```
pub struct Delete {
    pub input: Key<DataFlow>,

    /// The table reference to delete with.
    pub reference: RecordField,
    pub table: Key<Table>,

    /// `INV`: `[Self::input].with == output.with`
    pub output: Key<DataFlow>,
}

/// Borrow a field and use it for unique lookup into a table, to get a row
/// reference.
/// - The column used for the row lookup must have the unique constraint
///
/// ```text
/// RECORD [-> or |>] unique_ref(use RECORD.{ .. } TABLE at COLUMN ) [-> or |>] RECORD + TABLE::REF
/// ```
pub struct UniqueRef {
    pub input: Key<DataFlow>,

    pub from: RecordField,
    pub table: Key<Table>,
    pub field: RecordField,
    pub out: RecordField,

    pub output: Key<DataFlow>,
}

/// Scan all refs from a table into a stream.
///
/// ```text
/// scan_refs(TABLE) -> TABLE::REF
/// ```
pub struct ScanRefs {
    pub table: Key<Table>,
    pub out_ref: RecordField,

    //. `INV`: is a stream with a single field (`out_ref`) of table reference to `table`
    pub output: Key<DataFlow>,
}

/// Dereference a table reference and place in a variable
/// - `INV`: the 'named' not present in the input record
pub struct DeRef {
    pub input: Key<DataFlow>,
    pub reference: RecordField,
    pub named: RecordField,
    pub table: Key<Table>,
    pub output: Key<DataFlow>,
}

/// Applying a function over a stream of values
/// - `INV`: output fields match mapping fields
/// - `INV`: mapping expressions only contain fields from input and globals
pub struct Map {
    pub input: Key<DataFlow>,
    pub mapping: Vec<(RecordField, Expr)>,
    pub output: Key<DataFlow>,
}

/// Expand a single record field and discard the other fields.
/// - `INV`: field is in inputs, and is a record.
/// - `INV: output fields is `inputs.field`
pub struct Expand {
    pub input: Key<DataFlow>,
    pub field: RecordField,
    pub output: Key<DataFlow>,
}

pub struct FoldField {
    pub initial: Expr,
    pub update: Expr,
}

/// A fold operation over a stream of values
/// - `INV`: initial fields only contain globals
/// - `INV`: update expressions only contain fields from input, initial and globals
/// - `INV`: output matches initial types
pub struct Fold {
    pub input: Key<DataFlow>,
    pub fold_fields: Vec<(RecordField, FoldField)>,
    pub output: Key<DataFlow>,
}

/// Filter a stream of values
/// - `INV`: predicate expression only contains fields from input and globals
pub struct Filter {
    pub input: Key<DataFlow>,
    pub predicate: Expr,
    pub output: Key<DataFlow>,
}

pub enum SortOrder {
    Asc,
    Desc,
}

/// Sort the input given some keys and ordering
/// - `INV`: input and output must have the same fields
/// - `INV`: input and output must both be streams
/// - `INV`: The identified fields must exist in the input
pub struct Sort {
    pub input: Key<DataFlow>,
    pub sort_order: Vec<(RecordField, SortOrder)>,
    pub output: Key<DataFlow>,
}

/// Assert a boolean expression over a stream, or single value
/// - `INV`: input type is same as output type
/// - `INV`: predicate expression only contains fields from input and globals
pub struct Assert {
    pub input: Key<DataFlow>,
    pub assert: Expr,
    pub output: Key<DataFlow>,
}

/// A fold that outputs a collection of all data input, included here to allow
/// the optimiser to reason more easily about the data structure size & type
/// given many queries collect multiple rows.
/// - Generates the [`super::ScalarTypeConc::Bag`] types
/// - Could technically be implemented as a fold, but more difficult to reason
///   about folds generally, and we want the backend to determine the bag's type
pub struct Collect {
    pub input: Key<DataFlow>,
    pub into: RecordField,
    pub output: Key<DataFlow>,
}

/// Take the top n from a stream, discarding the rest
pub struct Take {
    pub input: Key<DataFlow>,
    pub top_n: Expr,
    pub output: Key<DataFlow>,
}

pub enum MatchKind {
    Cross,
    Pred(Expr),
    Equi {
        left_field: RecordField,
        right_field: RecordField,
    },
}

// TODO: Add more join kinds (left, outer), this is partially waiting on being
//       able to wrap emdb type in rust types, e.g. left join producing (left, Option<right>)
/// The join type of the operator
pub enum JoinKind {
    Inner,
}

/// Join two streams together
pub struct Join {
    pub left: Key<DataFlow>,
    pub right: Key<DataFlow>,
    pub match_kind: MatchKind,
    pub join_kind: JoinKind,
    pub output: Key<DataFlow>,
}

/// Group by a field and aggregate the results
pub struct GroupBy {
    pub input: Key<DataFlow>,

    pub group_by: RecordField,
    pub stream_in: Key<DataFlow>,
    pub inner_ctx: Key<Context>,

    pub output: Key<DataFlow>,
}

/// Run a sub-query for each row in an input stream
pub struct ForEach {
    pub input: Key<DataFlow>,

    pub stream_in: Key<DataFlow>,
    pub inner_ctx: Key<Context>,

    pub output: Key<DataFlow>,
}

/// Given an operator output, multiply it into multiple outputs
pub struct Fork {
    pub input: Key<DataFlow>,
    pub outputs: Vec<Key<DataFlow>>,
}

/// Merge a number of streams into one
/// - `INV`: All incomping dataflows are streams with the same type index
pub struct Union {
    pub inputs: Vec<Key<DataFlow>>,
    pub output: Key<DataFlow>,
}

/// Generate a single row
/// - `INV`: output matches fields
/// - `INV`: output is a single
pub struct Row {
    pub fields: Vec<(RecordField, Expr)>,
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
#[enumtrait::store(pub operator_enum)]
pub enum Operator {
    // get references
    UniqueRef,
    ScanRefs,

    // read operator
    DeRef,

    // write operators
    Update,
    Insert,
    Delete,

    // pure operators
    Map,
    Expand,
    Fold,
    Filter,
    Sort,
    Assert,

    // cardinality set
    Take,
    Collect,

    // nested contexts
    GroupBy,
    ForEach,

    // stream join & split
    Join,
    Fork,
    Union,

    // control flow
    Row,
    Return,
    Discard,
}

impl Operator {
    /// Convenience for getting a return operator (e.g. from a [Context])
    /// - `INV`: the operator is a return operator
    pub fn get_return(&self) -> &Return {
        match self {
            Operator::Return(r) => r,
            _ => unreachable!("Attempted to get return from non-return operator"),
        }
    }
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
