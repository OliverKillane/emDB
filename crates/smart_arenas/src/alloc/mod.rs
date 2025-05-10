use crate::id::index::{Index, WidestIndex};

mod blocks;
mod contig;

pub use blocks::*;
pub use contig::*;

pub trait AllocSelect {
    type Impl<Idx: Index, Data>: AllocImpl<Idx, Data>;
}

/// ## Allocators
/// Allowing [crate::arena] to allocate slots with easily configurable allocators.
///  - Custom, or structures using the global allocator.
///  - provide append only, with incrementing keys.
pub trait AllocImpl<Idx: Index, Data> {
    fn new(preallocate_to: Idx) -> Self;

    /// # Safety
    /// Must be deterministic, each next index is an increment, starting from [Index::ZERO]
    ///  - This is relied upon by the [crate::prelude::TransformArena] implementation.
    fn append(&mut self, d: Data) -> Option<Idx>;

    /// Provide the next key to allocate.
    ///
    /// # Safety
    /// Must match exactly the next key produced by insert
    fn next(&self) -> Option<Idx>;

    /// # Safety
    /// The index must have been allocated by [AllocImpl::append]
    unsafe fn read(&self, idx: Idx) -> &Data;

    /// # Safety
    /// The index must have been allocated by [AllocImpl::append]
    unsafe fn write(&mut self, idx: Idx) -> &mut Data;

    fn exclusive_index_upper_bound(&self) -> WidestIndex;
    fn is_empty(&self) -> bool {
        self.exclusive_index_upper_bound() == 0
    }
}
