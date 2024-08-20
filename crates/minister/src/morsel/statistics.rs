//! ## Statistics
//! Used for both compile & runtime optimisation
//! - Morsel driven parallelism is benefitted by knowing how to split work well
//!   (requires runtime statistics)
//! - Collection of data into buffers benefits from cardinality estimates

use std::ops::Range;

/// Cardinality constraints for an operator.
pub enum Cardinality {
    Exact(usize),
    Bound (Range<usize>),
    Unknown,
}

pub const fn reduce(card: Cardinality) -> Cardinality {
    match card {
        Cardinality::Exact(c) => Cardinality::Bound(0..c),
        c => c,
    }
}

pub struct Estimate {
    /// The estimated size of the data (even unknown cardinality needs some kind
    /// of estimate, for example using previous statistics)
    pub size: usize,

    /// An arbitrary measure of work
    /// NOTE: In future this should use timings from statistics
    pub work: usize,

    /// `0`` is perfect confidence, `usize::MAX` is no confidence
    pub confidence: usize,
}

