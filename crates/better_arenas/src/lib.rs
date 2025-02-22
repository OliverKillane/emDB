pub mod drop;
pub mod unique;
pub mod arenas;
pub mod slots;
pub mod ints;

pub trait RcArena<Data> {
    type Index;

    fn new() -> Self;

    fn brw(&self, idx: &Self::Index) -> &Data;
    fn brw_mut(&self, idx: &Self::Index) -> &mut Data;
    fn insert(&mut self, data: Data) -> Self::Index;

    fn inc(&self, idx: &Self::Index) -> Self::Index;
    fn dec(&self, idx: Self::Index);
}