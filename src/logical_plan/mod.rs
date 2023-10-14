//! The logical plan data structure for emDB
use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};
use syn::Type;
use typed_arena::Arena;

// TODO: consider interning these for better perf
enum DataType {
    /// A series of data, streamed through operators.
    Stream(Box<DataType>),

    /// A fixed size bucket of rows
    /// - Can convert streams to pass buckets of multiple items at once
    Bucket { size: usize, data: Box<DataType> },

    /// A normal database row, propagated through operators
    Row(Vec<DataType>),

    /// It is a reference, for use in ref streams (e.g inserts to a table).
    /// - Not necessarily just a rust reference (e.g a LockGuard, borrow of some
    ///   string type reference)
    /// - Can reference a table, or an intermediate materialised table.
    Ref(Box<DataType>),

    /// A Rust Type
    Type(Type),
}

/// ## Contains the entire logical plan.
/// Each node in the graph is represented by one of the node types ([StreamOperator],
/// [CallOperator], [Table] etc). With nodes encapsulated in read-write locks
/// (for single ownership).
/// - References allow easier access from just the node (rather than needing the
///   whole plan as with indexes)
/// - Nodes can constrain number of connected nodes and use different types of
///   nodes (not possible with [Petgraph](https://github.com/petgraph/petgraph))
/// - intended to be concurrency ready (hence [RwLock] rather than [RefCell])
/// - Nodes are never deleted, but instead marked removed for reuse.
///
/// TODO: Visitor Pattern / Traverser
/// TODO: make printable as dots (for graphviz)
struct LogicalPlan<'a> {
    stream_operators: Arena<RwLock<StreamOperator<'a>>>,
    single_operators: Arena<RwLock<SingleOperator<'a>>>,
    var_single: Arena<VarSingle>,
    methods: Arena<CallOperator<'a>>,
    tables: Arena<RwLock<Table>>,
}

struct VarSingle {
    datatype: DataType,
}

/// The return value of a Call Operator
/// - Some stream returns can be optimised into references, or lock guards.
/// - Some need to collect to a single.
enum ReturnVal<'a> {
    None,
    Single(&'a RwLock<SingleOperator<'a>>),
    Stream(&'a RwLock<StreamOperator<'a>>),
}

struct CallOperator<'a> {
    name: String,
    args: HashMap<String, &'a VarSingle>,
    returnval: ReturnVal<'a>,
}

struct Col {
    name: String,
    datatype: DataType,
}

type Data = ();

struct Constraint {}

struct Table {
    cols: Vec<Col>,
    initialdata: Vec<Vec<Data>>,
    constraints: HashSet<Constraint>,
}

enum StreamOperator<'a> {
    Scan {
        source: &'a RwLock<Table>,
    },
    Map {
        prev: &'a RwLock<StreamOperator<'a>>,
        func: (),
    },
    Sort {
        prev: &'a RwLock<StreamOperator<'a>>,
        col: (),
        sort: (),
    },
    Join {
        left: &'a RwLock<StreamOperator<'a>>,
        right: &'a RwLock<StreamOperator<'a>>,
        cond: (),
    },
    Limit {
        prev: &'a RwLock<StreamOperator<'a>>,
        vol: (),
    },
    Filter {
        prev: &'a RwLock<StreamOperator<'a>>,
        cond: (),
    },

    /// Allows users to repeat a stream a given number of times
    /// - `1,2,3` through repeat(2) becomes `1,1,2,2,3,3`
    Repeat {
        prev: &'a RwLock<SingleOperator<'a>>,
        times: (),
    },

    /// Groups by a given column identifier from a row stream
    /// - breaks the pipeline breaker (partially)
    GroupBy {
        prev: &'a RwLock<SingleOperator<'a>>,
        col: (),
    },

    /// Appends two streams of the same type
    Union {
        left: &'a RwLock<StreamOperator<'a>>,
        right: &'a RwLock<StreamOperator<'a>>,
    },

    /// Converts a stream of values into a temporary table from which references
    /// can be streamed
    Materialise {
        prev: &'a RwLock<SingleOperator<'a>>,
    },

    /// Indicates the operator was removed
    Removed,
}

enum SelectFunc {
    Max,
    Min,
    Median,
    First(Predicate),
    Last(Predicate),
}

enum SingleOperator<'a> {
    Unique {
        prev: &'a RwLock<StreamOperator<'a>>,
        col: (),
        select: (),
    },
    Select {
        prev: &'a RwLock<StreamOperator<'a>>,
        col: (),
        func: SelectFunc,
    },
    VarInput {
        prev: &'a VarSingle,
    },

    /// Allows the user to constrain types
    Fold {
        prev: &'a RwLock<StreamOperator<'a>>,
        collect_fun: (),
    },

    Removed,
}
