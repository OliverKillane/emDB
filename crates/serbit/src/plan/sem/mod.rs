use crate::plan::ir;

pub mod recur;
pub mod order;
pub mod unions;
pub mod name;

struct SemError;

trait Semantic {
    type Error: Into<SemError>;

    fn analyse<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints>(
        plan: &ir::Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, impl ir::Naming>,
    ) -> Result<(), Vec<Self::Error>>;
}