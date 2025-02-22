//! ## Interface Agnostic Data Structures for [super::arenas].

use crate::ints::IdxInt;

pub mod blocks;
pub mod contig;
pub mod amort;

/// A simple interface for data structures holding values, with keys chosen by the structure.
pub trait Slots<Idx: IdxInt, Data> {
    type Cfg;
    fn new(cfg: Self::Cfg) -> Self;
    fn insert(&mut self, d: Data) -> Option<Idx>;
    unsafe fn read(&self, idx: Idx) -> &Data;
    unsafe fn write(&mut self, idx: Idx) -> &mut Data;
}
