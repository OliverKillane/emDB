use crate::{
    access,
    data::Data,
    utils::{
        size::{Bits, bytes},
        truth::{Bool, Truth},
    },
};

pub mod buffer;

/// A movable span into some data structure.
///  - Includes compile time bounds [Cursor::Bound] for eliminating bounds
///    checks at compile time know locations
///  - Does not leak the internal state (can be backed by a buffer, lazy structure, etc.)
pub trait Cursor {
    type Bound: Bound;
}

pub trait Reader: Cursor {
    fn read<P: access::CompTime>(&self) -> P::Data
    where
        P: CheckAccess<Self::Bound>,
        [(); bytes(P::Data::SIZE)]:;
    fn try_read<P: access::RunTime>(&self, pos: &P) -> Option<P::Data>
    where
        [(); bytes(P::Data::SIZE)]:;
}

pub trait Advance: Cursor {
    fn advance<NEW: Bound, const BITS: usize>(&self) -> impl Cursor<Bound = NEW>
    where
        Self: CheckAdvance<NEW, BITS>;
    fn try_advance<NEW: Bound>(&self, bits: Bits) -> Option<impl Cursor<Bound = NEW>>;
}

// TODO: Implement write, and split (for splitting mutable references)

pub trait Bound {
    const SIZE: Bits;
}

pub unsafe trait CheckAdvance<B: Bound, const BY: Bits> {}
pub unsafe trait CheckAccess<B: Bound> {}

unsafe impl<A: access::CompTime, B: Bound> CheckAccess<B> for A where
    Bool<{ A::OFFSET + (A::Data::SIZE * (A::INDEX + 1)) < B::SIZE }>: Truth
{
}

unsafe impl<B: Bound, C: Cursor, const BY: Bits> CheckAdvance<B, BY> for C where
    Bool<{ BY + B::SIZE <= C::Bound::SIZE }>: Truth
{
}
