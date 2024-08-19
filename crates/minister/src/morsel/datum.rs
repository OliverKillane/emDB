use std::{cell::UnsafeCell, mem::MaybeUninit, sync::Arc};

use super::splitter::Splitter;

/// A wrapper with debug assertions for accessing data in arrays unsafely.
/// - Data must be present
/// - Data must be used before destruction.
pub struct Datum<T> {
    #[cfg(debug_assertions)]
    state: DatumState,
    data: MaybeUninit<T>,
}

impl<T> Datum<T> {
    #[inline(always)]
    pub fn new(data: T) -> Self {
        Self {
            #[cfg(debug_assertions)]
            state: DatumState::Present,
            data: MaybeUninit::new(data),
        }
    }

    #[inline(always)]
    pub fn new_empty() -> Self {
        Self {
            #[cfg(debug_assertions)]
            state: DatumState::Uninitialised,
            data: MaybeUninit::uninit(),
        }
    }

    #[inline(always)]
    pub fn get(&mut self) -> T {
        #[cfg(debug_assertions)]
        {
            assert_eq!(self.state, DatumState::Present);
            self.state = DatumState::Consumed;
        }
        unsafe { self.data.assume_init_read() }
    }

    #[inline(always)]
    pub fn put(&mut self, data: T, prev_state: DatumState) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(self.state, prev_state);
            self.state = DatumState::Present;
        }

        if let DatumState::Present = prev_state {
            unsafe { self.data.assume_init_drop() };
        }

        self.data.write(data);
    }
}

impl <T> Drop for Datum<T> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            // NOTE: Really this should be an assertion that none are present, 
            //       however panicking in a drop is bad as drop is also called 
            //       on unwind (from other panics)
            if let DatumState::Present = self.state {
                unsafe { self.data.assume_init_drop() };
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum DatumState {
    /// Uninitialised memory
    Uninitialised,

    /// Owned data is present
    Present,

    /// Data has been removed, but memory still set
    Consumed,
}
