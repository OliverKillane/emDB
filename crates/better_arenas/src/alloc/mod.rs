//! ## Interface Agnostic Data Structures for [super::arenas].

use crate::utils::idx::IdxInt;

pub mod amort;
pub mod blocks;
pub mod contig;

pub trait AllocSelect {
    type Impl<Idx: IdxInt, Data>: AllocImpl<Idx, Data>;
}

/// A simple interface for data structures holding values, with keys chosen by the structure.
pub trait AllocImpl<Idx: IdxInt, Data> {
    type Cfg;
    fn new(cfg: Self::Cfg) -> Self;
    fn insert(&mut self, d: Data) -> Option<Idx>;
    unsafe fn read(&self, idx: Idx) -> &Data;
    unsafe fn write(&mut self, idx: Idx) -> &mut Data;
}
