//! ## Statistics
//! Used for both compile & runtime opetimisation
//! - Morsel driven parallelism is benefitted by knowing how to split work well
//!   (requires runtime statistics)
//! - Collection of data into buffers benefits from cardinality estimates

pub enum Size {
    Exact(usize),
    Gte(usize),
    Lte(usize),
    UnKnown,
}

pub enum Cardinality {
    Exact(Size),
    Range {
        upper: Size,
        estimate: Size,
        lower: Size,
    },
}
