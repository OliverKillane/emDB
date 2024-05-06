//! # Get copies of immutable values.
//! Used for small types for which a copy is better than returning a reference to dereference.
//!
//! This also means that pointer stability can be ignored (no references, just copies)

use super::{ImmutVal, Pullable, StableImmutVal, UnStableImmutVal};

pub struct ValCpy<Val> {
    data: Val,
}

impl<'db, Store> ImmutVal<'db, Store> for ValCpy<Store>
where
    Store: 'db + Clone,
{
    type Get = Store;
    type Brw<'a> = &'a Store where Self: 'a;

    #[inline(always)]
    fn from_store(s: Store) -> Self {
        ValCpy { data: s }
    }

    #[inline(always)]
    fn brw(&self) -> Self::Brw<'_> {
        &self.data
    }
}

impl<'db, Store> StableImmutVal<'db, Store> for ValCpy<Store>
where
    Store: 'db + Clone,
{
    #[inline(always)]
    fn get(&self) -> Self::Get {
        self.data.clone()
    }
}

impl<'db, Store> UnStableImmutVal<'db, Store> for ValCpy<Store>
where
    Store: 'db + Clone,
{
    type GetAux = ();

    #[inline(always)]
    fn get(&self, _aux: Self::GetAux) -> Self::Get {
        self.data.clone()
    }
}

unsafe impl<'db, Store> Pullable<'db, Store> for ValCpy<Store>
where
    Store: 'db + Clone,
{
    type Own = Store;

    #[inline(always)]
    fn pull(&mut self) -> Self::Own {
        self.data.clone()
    }

    fn place(&mut self, s: Self::Own) {
        self.data = s;
    }
}
