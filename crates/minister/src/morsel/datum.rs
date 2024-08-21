use std::{cell::UnsafeCell, mem::MaybeUninit, ops::Deref, sync::Arc};

// NOTE: Required to allow for in-place collection to make conversion between
//       `Datum<T>` and `T` zero cost/eliminatable (especially important when
//       converting between `Vec<T>` and `Vec<Datum<T>>`)
const _: () = {
    use std::mem::size_of;
    let (datum_size, empty_size) = (size_of::<Datum<()>>(), size_of::<()>());

    if {
        #[cfg(not(debug_assertions))]
        {
            datum_size != empty_size
        }

        #[cfg(debug_assertions)]
        {
            datum_size != empty_size + size_of::<DatumState>()
        }
    } {
        panic!("Without debug implementation `Datum<T>` must be same size as `T`")
    }
};

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

impl<T> Drop for Datum<T> {
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

/// In order to avoid nightly usage ([`SyncUnsafeCell`] is `!Sync` to reduce accidental misuse)
/// we need to wrap our own version.
/// TODO: pressure [issue #95439](https://github.com/rust-lang/rust/issues/95439)
pub struct SyncUnsafeCellWrap<T>(pub UnsafeCell<T>);

impl<T> Deref for SyncUnsafeCellWrap<T> {
    type Target = UnsafeCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T: Sync> Sync for SyncUnsafeCellWrap<T> {}
