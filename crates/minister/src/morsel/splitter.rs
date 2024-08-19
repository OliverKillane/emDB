//! ## Splitter
//! An object for dividing wortk using statistics.

use std::{iter::once, ops::Range};

pub trait Splitter {
    fn split_ranges(&self, units: Range<usize>) -> impl Iterator<Item = Range<usize>>;
    fn split_slice<'me, 'buffer: 'me, Data>(&'me self, slice: &'buffer [Data]) -> impl Iterator<Item = &'buffer [Data]> + 'me;
    fn split_slice_mut<'me, 'buffer: 'me, Data>(&'me self, slice: &'buffer mut [Data]) -> impl Iterator<Item = &'buffer mut [Data]> + 'me;
}

pub struct SingleSplit;

impl Splitter for SingleSplit {
    fn split_ranges(&self, units: Range<usize>) -> impl Iterator<Item = Range<usize>> {
        once(units)
    }

    fn split_slice<'me, 'buffer: 'me, Data>(&'me self, slice: &'buffer [Data]) -> impl Iterator<Item = &'buffer [Data]> + 'me {
        once(slice)
    }

    fn split_slice_mut<'me, 'buffer: 'me, Data>(&'me self, slice: &'buffer mut [Data]) -> impl Iterator<Item = &'buffer mut [Data]> + 'me {
        once(slice)
    }
}
