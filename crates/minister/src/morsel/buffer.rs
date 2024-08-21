//! ## Buffers
//! Nodes in the physical plan which can provide morsels

use std::{cell::UnsafeCell, ops::Range, sync::Arc};
use super::{datum::{Datum, SyncUnsafeCellWrap}, splitter::Splitter};

pub struct BufferSpan<Data> {
    span: Range<usize>,
    arr: Arc<[SyncUnsafeCellWrap<Datum<Data>>]>,
}

impl<Data> From<Vec<Data>> for BufferSpan<Data> {
    #[inline(always)]
    fn from(value: Vec<Data>) -> Self {
        let wrapped_data = value
            .into_iter()
            .map(|d| SyncUnsafeCellWrap(UnsafeCell::new(Datum::new(d))))
            .collect::<Vec<_>>();
        let arr: Arc<[SyncUnsafeCellWrap<Datum<Data>>]> = Arc::from(wrapped_data.into_boxed_slice());
        Self {
            span: Range::from(0..arr.len()),
            arr,
        }
    }
}

impl<Data> BufferSpan<Data> {
    #[inline(always)]
    pub fn split(self, splitter: &impl Splitter) -> impl Iterator<Item = Self> {
        splitter
            .split_ranges(self.span)
            .map(|range| Self {
                span: range,
                arr: self.arr.clone(),
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.span.len()
    }
}

impl<Data> Iterator for BufferSpan<Data> {
    type Item = Data;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.span.next() {
            let cell: &UnsafeCell<Datum<Data>> = unsafe { self.arr.get_unchecked(index) };
            let datum: &mut Datum<Data> = unsafe { &mut *cell.get() };
            let data = datum.get();
            Some(data)
        } else {
            None
        }
    }
}
