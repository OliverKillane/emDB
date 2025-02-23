use std::mem::MaybeUninit;

use crate::utils::idx::IdxInt;

use super::AllocImpl;

/// Allocates in stable blocks, doubling size of each consecutive block.
///  - Initial block size based on element.
///  - Can use msb to work out the block used.
struct AmortImpl<Idx: IdxInt, Data, const FIRST_BLOCK_SIZE: usize> {
    data: Vec<Box<[MaybeUninit<Data>]>>,
    last_idx: Option<Idx>,
}

impl<Idx: IdxInt, Data, const FIRST_BLOCK_SIZE: usize> AllocImpl<Idx, Data>
    for AmortImpl<Idx, Data, FIRST_BLOCK_SIZE>
{
    type Cfg = ();
    fn new(cfg: Self::Cfg) -> Self {
        todo!()
    }

    fn insert(&mut self, d: Data) -> Option<Idx> {
        todo!()
    }

    unsafe fn read(&self, idx: Idx) -> &Data {
        todo!()
    }

    unsafe fn write(&mut self, idx: Idx) -> &mut Data {
        todo!()
    }
}
