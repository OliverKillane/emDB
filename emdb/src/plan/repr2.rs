use std::collections::HashMap;

use enumset::{EnumSet, EnumSetType};
use proc_macro2::Ident;
use syn::{Expr, Type};
use typed_generational_arena::{Arena as GenArena, Index as GenIndex};

type ComplexCons<T> = Option<(T, Option<Ident>)>;
type PresentCons = ComplexCons<()>;

struct LogicalColumnConstraint {
    read: PresentCons,
    write: PresentCons,
    unique: PresentCons,
}

struct LogicalRowConstraint {
    insert: PresentCons,
    delete: PresentCons,
    limit: ComplexCons<Expr>,
    genpk: ComplexCons<Ident>,
    preds: Vec<(Expr, Option<Ident>)>,
}

struct LogicalColumn {
    name: Ident,
    constraints: LogicalColumnConstraint,
    data_type: Type,
}

struct LogicalTable {
    name: Ident,
    constraints: LogicalRowConstraint,
    columns: HashMap<Ident, GenIndex<LogicalColumn>>,
}

enum RecordData {
    Record(Record),
    Ref(GenIndex<LogicalTable>),
    Rust(Type),
}
struct Record {
    fields: HashMap<Ident, RecordData>,
}

struct Edge {
    from: GenIndex<Operator>,
    to: GenIndex<Operator>,
    stream: bool,
    with: Record,
}

type EdgeKey = GenIndex<Edge>;

enum Operator {
    /// Apply a series of updates from a stream
    ///
    Update {
        input: EdgeKey,
        reference: Expr,
        mapping: HashMap<Ident, (Type, Expr)>,
        output: EdgeKey,
    },

    /// Applying a function over a stream of values
    /// INV: output fields match mapping fields
    /// INV: mapping expressions only contain fields from input and globals
    Map {
        input: EdgeKey,
        mapping: HashMap<Ident, (Type, Expr)>,
        output: EdgeKey,
    },

    /// Filter a stream of values
    /// INV: predicate expression only contains fields from input and globals
    Filter {
        input: EdgeKey,
        predicate: Expr,
        output: EdgeKey,
    },

    Row {
        
    }

    ///
    Unique {
        refs: bool,
        table: (),
        output: EdgeKey,
    },

    Scan {
        refs: bool,
        table: (),
        output: EdgeKey,
    },

    Sort {
        input: EdgeKey,
        output: EdgeKey,
    },
}

///
pub struct LogicalPlan {
    columns: GenArena<LogicalColumn>,
    tables: GenArena<LogicalTable>,
    record_types: GenArena<Record>,
    operators: GenArena<Operator>,
    stream_edges: GenArena<Edge>,
    single_edges: GenArena<Edge>,
}
