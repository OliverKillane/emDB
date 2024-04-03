use typed_generational_arena::{Index, NonzeroGeneration, StandardArena as GenArena};

mod operators;
mod queries;
mod tables;
mod types;

pub use operators::*;
pub use queries::*;
pub use tables::*;
pub use types::*;

/// All component types can be indexed through a [Key]
/// - No shared mutability, need to have the plan also to use
/// - Checked access for keys to ensure no use after delete
/// - Keys are generational, so no aliasing of old deleted, vs new keys is
///   possible.
pub type Key<T> = Index<T, usize, NonzeroGeneration<usize>>;

/// A wrapper type for implementing traits on components that need to use the
/// plan for context.
/// - for example printing types requires the logical plan for table ref types
pub struct With<'a, A> {
    pub plan: &'a Plan,
    pub extended: A,
}

/// The basic logical plan
/// - All components can be accessed via [Key]
/// - Can be agumented with other data that uses [Key] to reference components
pub struct Plan {
    pub queries: GenArena<Query>,
    pub tables: GenArena<Table>,
    pub operators: GenArena<Operator>,
    pub operator_edges: GenArena<DataFlow>,
    pub scalar_types: GenArena<ScalarType>,
    pub record_types: GenArena<Record>,
}

impl Plan {
    pub fn new() -> Self {
        Plan {
            queries: GenArena::new(),
            tables: GenArena::new(),
            operators: GenArena::new(),
            operator_edges: GenArena::new(),
            scalar_types: GenArena::new(),
            record_types: GenArena::new(),
        }
    }
}
