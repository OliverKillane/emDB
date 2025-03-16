use std::marker::PhantomData;

use crate::utils::idx::IdxInt;

use super::{AllocImpl, AllocSelect};

pub struct ContigConfig<Idx: IdxInt> {
    pub preallocate_to: Idx,
}

/// A continugous allocation of slots.
///  - Backed by a vector.
///  - Copies entire vector on resizes that require
pub struct ContigImpl<Idx: IdxInt, Data> {
    data: Vec<Data>,
    _phantom: PhantomData<Idx>,
}

pub struct Contig;

impl AllocSelect for Contig {
    type Impl<Idx: IdxInt, Data> = ContigImpl<Idx, Data>;
}

impl<Idx: IdxInt, Data> AllocImpl<Idx, Data> for ContigImpl<Idx, Data> {
    type Cfg = ContigConfig<Idx>;

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
        unsafe { self.data.get_unchecked(idx.offset()) }
    }

    unsafe fn write(&mut self, idx: Idx) -> &mut Data {
        unsafe { self.data.get_unchecked_mut(idx.offset()) }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}
