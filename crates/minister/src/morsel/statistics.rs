//! ## Statistics
//! Used for both compile & runtime optimisation
//! - Morsel driven parallelism is benefitted by knowing how to split work well
//!   (requires runtime statistics)
//! - Collection of data into buffers benefits from cardinality estimates

use std::ops::Range;

/// Cardinality constraints for an operator.
pub enum Cardinality {
    Exact(usize),
    Bound(Range<usize>),
    Unknown,
}

pub const fn reduce(card: Cardinality) -> Cardinality {
    match card {
        Cardinality::Exact(c) => Cardinality::Bound(0..c),
        c => c,
    }
}

pub const fn combine(left: Cardinality, right: Cardinality) -> Cardinality {
    // NOTE: `(L, R) | (R, L)` pattern, versus `(L, R) => .., (l, r) => recur(r, l)` 
    //       to swap. Decided to avoid (one level of) recursion/style preference.
    match (left, right) {
        (Cardinality::Unknown, _) | (_, Cardinality::Unknown) => Cardinality::Unknown,
        (Cardinality::Exact(l), Cardinality::Exact(r)) => Cardinality::Exact(l + r),
        (Cardinality::Bound(l), Cardinality::Exact(r))
        | (Cardinality::Exact(r), Cardinality::Bound(l)) => {
            Cardinality::Bound(l.start + r..l.end + r)
        }
        (Cardinality::Bound(l), Cardinality::Bound(r)) => {
            Cardinality::Bound(l.start + r.start..l.end + r.end)
        }
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

// TODO: placeholder
pub fn union_estimates(left: Estimate, right: Estimate ) -> Estimate {
    Estimate {
        size: left.size + right.size,
        work: left.work + right.work,
        confidence: left.confidence + right.confidence,
    }
}


/// Consider range but only integers, and supporting splits better, and infinite enableable
struct Range2<const CONCRETE: bool> {}