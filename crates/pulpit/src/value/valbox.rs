//! # A Value to convert stable values to support unstable columns.
//! A wrapper around [`Box`]
//!
//! ## When to Use?
//! The additional costs of Box are:
//! - Allocation (we can use a different allocator)
//! - Double Dereference (when getting items)
//!
//! However in an unstable column, the items do not need to actually be moved.
//! Hence the values are actually stable, and can be shared as if part of a
//! stable column.
//!
//! We can use this to convert [`StableImmutVal`] into [`UnStableImmutVal`]

use super::{ImmutVal, StableImmutVal, UnStableImmutVal};
use std::{marker::PhantomData, pin::Pin};

pub struct ValBox<'db, Val, Con: StableImmutVal<'db, Val>> {
    data: Pin<Box<Con>>,
    _phantomdata: PhantomData<&'db Val>,
}

impl<'db, Val, Con: StableImmutVal<'db, Val> + Unpin> ImmutVal<'db, Val> for ValBox<'db, Val, Con> {
    type Get = Con::Get;
    type Brw<'a> = Con::Brw<'a> where Self: 'a;

    #[inline(always)]
    fn from_store(s: Val) -> Self {
        ValBox {
            data: Pin::new(Box::new(Con::from_store(s))),
            _phantomdata: PhantomData,
        }
    }

    #[inline(always)]
    fn brw(&self) -> Self::Brw<'_> {
        self.data.brw()
    }
}

impl<'db, Val, Con: StableImmutVal<'db, Val> + Unpin> UnStableImmutVal<'db, Val>
    for ValBox<'db, Val, Con>
{
    type GetAux = ();

    #[inline(always)]
    fn get(&self, _aux: Self::GetAux) -> Self::Get {
        self.data.get()
    }
}
