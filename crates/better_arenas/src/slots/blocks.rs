use super::Slots;
use crate::ints::IdxInt;
use smallvec::SmallVec;
use std::{marker::PhantomData, mem::MaybeUninit};

struct Cfg<Idx: IdxInt> {
    preallocate_to: Idx,
}

/// Allocating slots in blocks.
///  - No reacllocation on extension.
///  - Each block is the same size.
struct Blocks<Idx: IdxInt, Data, const BLOCK_SIZE: usize> {
    data: SmallVec<[Box<[MaybeUninit<Data>; BLOCK_SIZE]>; 4]>,
    last_idx: Option<Idx>,
    _phantom: PhantomData<Idx>,
}
impl<Idx: IdxInt, Data, const BLOCK_SIZE: usize> Blocks<Idx, Data, BLOCK_SIZE> {
    fn idx_convert(idx: Idx) -> (usize, usize) {
        let block_idx = idx.offset() / BLOCK_SIZE;
        let inner_idx = idx.offset() % BLOCK_SIZE;
        (block_idx, inner_idx)
    }

    fn new_block() -> Box<[MaybeUninit<Data>; BLOCK_SIZE]> {
        Box::new([(); BLOCK_SIZE].map(|_| MaybeUninit::uninit()))
    }
}

impl<Idx: IdxInt, Data, const BLOCK_SIZE: usize> Slots<Idx, Data>
    for Blocks<Idx, Data, BLOCK_SIZE>
{
    type Cfg = Cfg<Idx>;

    fn new(cfg: Self::Cfg) -> Self {
        let blocks = cfg.preallocate_to.offset() / BLOCK_SIZE;
        let mut data = SmallVec::with_capacity(blocks as usize);
        for _ in 0..blocks {
            data.push(Self::new_block());
        }
        Self {
            data,
            last_idx: None,
            _phantom: PhantomData,
        }
    }

    fn insert(&mut self, d: Data) -> Option<Idx> {
        let maybe_last_idx = self.last_idx;
        let insert = |new_idx| {
            let (block, inner) = Self::idx_convert(new_idx);
            debug_assert!(block <= self.data.len());
            if block == self.data.len() {
                self.data.push(Self::new_block());
            }
            unsafe {
                self.data
                    .get_unchecked_mut(block)
                    .get_unchecked_mut(inner)
                    .write(d);
            }
            self.last_idx = Some(new_idx);
            new_idx
        };

        if let Some(last_idx) = maybe_last_idx {
            if last_idx == Idx::MAX {
                return None;
            } else {
                Some(insert(last_idx.inc()))
            }
        } else {
            Some(insert(Idx::MIN))
        }
    }

    unsafe fn read(&self, idx: Idx) -> &Data {
        let (block, inner) = Self::idx_convert(idx);
        unsafe {
            self.data
                .get_unchecked(block)
                .get_unchecked(inner)
                .assume_init_ref()
        }
    }

    unsafe fn write(&mut self, idx: Idx) -> &mut Data {
        let (block, inner) = Self::idx_convert(idx);
        unsafe {
            self.data
                .get_unchecked_mut(block)
                .get_unchecked_mut(inner)
                .assume_init_mut()
        }
    }
}
