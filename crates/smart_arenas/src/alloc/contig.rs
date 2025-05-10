use std::marker::PhantomData;

use crate::id::index::{Index, WidestIndex};

use super::{AllocImpl, AllocSelect};

/// A continugous allocation of slots.
///  - Backed by a vector.
///  - Copies entire vector on resizes that require
pub struct ContigImpl<Idx: Index, Data> {
    data: Vec<Data>,
    _phantom: PhantomData<Idx>,
}

pub struct Contig;

impl AllocSelect for Contig {
    type Impl<Idx: Index, Data> = ContigImpl<Idx, Data>;
}

impl<Idx: Index, Data> AllocImpl<Idx, Data> for ContigImpl<Idx, Data> {
    fn new(preallocate_to: Idx) -> Self {
        Self {
            data: Vec::with_capacity(preallocate_to.offset() as usize),
            _phantom: PhantomData,
        }
    }

    fn append(&mut self, d: Data) -> Option<Idx> {
        let idx = self.next();
        if idx.is_some() {
            self.data.push(d);
        }
        idx
    }

    fn next(&self) -> Option<Idx> {
        if <Idx as Index>::MAX.offset() as usize == self.data.len() {
            None
        } else {
            // JUSTIFY: Casting usize to u32
            //           - We cannot insert over the max offset, so we will never have a length
            //             larger than `u32::MAX`
            // JUSTIFY: Unwrapping the result
            //           - We only extend length when allocating, so this index was allocated, so it
            //             must be valid for the index type
            Some(Idx::from_offset(self.data.len() as u32).unwrap())
        }
    }

    unsafe fn read(&self, idx: Idx) -> &Data {
        unsafe { self.data.get_unchecked(idx.offset() as usize) }
    }

    unsafe fn write(&mut self, idx: Idx) -> &mut Data {
        unsafe { self.data.get_unchecked_mut(idx.offset() as usize) }
    }

    fn exclusive_index_upper_bound(&self) -> WidestIndex {
        self.data.len() as WidestIndex
    }
}
