use std::marker::PhantomData;

use crate::ints::IdxInt;

use super::Slots;

struct Cfg<Idx: IdxInt> {
    preallocate_to: Idx,
}

/// A continugous allocation of slots.
///  - Backed by a vector.
///  - Copies entire vector on resizes that require
struct Contig<Idx: IdxInt, Data> {
    data: Vec<Data>,
    _phantom: PhantomData<Idx>,
}

impl<Idx: IdxInt, Data> Slots<Idx, Data> for Contig<Idx, Data> {
    type Cfg = Cfg<Idx>;

    fn new(cfg: Self::Cfg) -> Self {
        Self {
            data: Vec::with_capacity(cfg.preallocate_to.offset()),
            _phantom: PhantomData,
        }
    }

    fn insert(&mut self, d: Data) -> Option<Idx> {
        if <Idx as IdxInt>::MAX.offset() == self.data.len() {
            None
        } else {
            self.data.push(d);
            // JUSTIFY: We never insert above the maximum size of the index,
            //          so this conversion cannot fail.
            Some(Idx::from_offset(self.data.len() - 1).unwrap())
        }
    }

    unsafe fn read(&self, idx: Idx) -> &Data {
        unsafe {
            self.data.get_unchecked(idx.offset())
        }
    }

    unsafe fn write(&mut self, idx: Idx) -> &mut Data {
        unsafe {
            self.data.get_unchecked_mut(idx.offset())
        }
    }
}
