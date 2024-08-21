//! ## Splitter
//! An object for dividing wortk using statistics.

use std::{iter::once, ops::Range};

use itertools::Itertools;

/// Divides work used for execution.
/// - On creation the number of
pub trait Splitter {
    fn create() -> Self;
    fn split_ranges(&self, units: Range<usize>) -> impl Iterator<Item = Range<usize>>;
    fn split_slice<'me, 'buffer: 'me, Data>(
        &'me self,
        slice: &'buffer [Data],
    ) -> impl Iterator<Item = &'buffer [Data]> + 'me;
    fn split_slice_mut<'me, 'buffer: 'me, Data>(
        &'me self,
        slice: &'buffer mut [Data],
    ) -> impl Iterator<Item = &'buffer mut [Data]> + 'me;
}

pub struct SingleSplit;

impl Splitter for SingleSplit {
    fn create() -> Self {
        SingleSplit
    }

    fn split_ranges(&self, units: Range<usize>) -> impl Iterator<Item = Range<usize>> {
        once(units)
    }

    fn split_slice<'me, 'buffer: 'me, Data>(
        &'me self,
        slice: &'buffer [Data],
    ) -> impl Iterator<Item = &'buffer [Data]> + 'me {
        once(slice)
    }

    fn split_slice_mut<'me, 'buffer: 'me, Data>(
        &'me self,
        slice: &'buffer mut [Data],
    ) -> impl Iterator<Item = &'buffer mut [Data]> + 'me {
        once(slice)
    }
}

pub struct EvenSplit {
    threads: usize,
}

impl Splitter for EvenSplit {
    fn create() -> Self {
        EvenSplit {
            threads: rayon::current_num_threads(),
        }
    }

    fn split_ranges(&self, units: Range<usize>) -> impl Iterator<Item = Range<usize>> {
        // NOTE: Efficient integer divide and round up
        let size = ((units.len() - 1) / self.threads) + 1;
        let end = units.end;
        units
            .step_by(size)
            .map(move |start| start..(start + size).min(end))
    }

    fn split_slice<'me, 'buffer: 'me, Data>(
        &'me self,
        slice: &'buffer [Data],
    ) -> impl Iterator<Item = &'buffer [Data]> + 'me {
        let size = ((slice.len() - 1) / self.threads) + 1;
        slice.chunks(size)
    }

    fn split_slice_mut<'me, 'buffer: 'me, Data>(
        &'me self,
        slice: &'buffer mut [Data],
    ) -> impl Iterator<Item = &'buffer mut [Data]> + 'me {
        let size = ((slice.len() - 1) / self.threads) + 1;
        slice.chunks_mut(size)
    }
}
