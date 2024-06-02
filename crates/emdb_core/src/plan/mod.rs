//! # emDB Logical Plan
//! Describes the schema, tables, expressions and operations.
//!
//! ## Shortcomings
//! Heavy usage of indexes, and not grouping queries, contexts, and operators optimally.
//! - More invariants checked and reasoned about, but not enforced by types.
//! - Potentially nesting contexts with dataflow and operators inside.

use typed_generational_arena::{Index, NonzeroGeneration, StandardArena as GenArena};

mod access;
mod operators;
mod queries;
mod tables;
mod types;

pub use access::*;
pub use operators::*;
pub use queries::*;
pub use tables::*;
pub use types::*;

/// The basic logical plan
/// - All components can be accessed via [Key]
/// - Can be agumented with other data that uses [Key] to reference components
#[allow(clippy::manual_non_exhaustive)]
pub struct Plan {
    pub queries: GenArena<Query>,
    pub contexts: GenArena<Context>,
    pub tables: GenArena<Table>,
    pub operators: GenArena<Operator>,
    pub dataflow: GenArena<DataFlow>,
    pub scalar_types: GenArena<ScalarType>,
    pub record_types: GenArena<RecordType>,
    _holder: (),
}

impl Plan {
    pub fn new() -> Self {
        Plan {
            queries: GenArena::new(),
            contexts: GenArena::new(),
            tables: GenArena::new(),
            operators: GenArena::new(),
            dataflow: GenArena::new(),
            scalar_types: GenArena::new(),
            record_types: GenArena::new(),
            _holder: (),
        }
    }
}
