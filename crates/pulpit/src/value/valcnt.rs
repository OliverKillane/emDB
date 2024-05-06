//! # Reference Counted Values
//! Stored separately from column and thus able to be shared safely in unstable columns.

use super::{ImmutVal, StableImmutVal, UnStableImmutVal};
use std::rc::Rc;

pub struct ValCnt<Val> {
    data: Rc<Val>,
}

impl<'db, Store> ImmutVal<'db, Store> for ValCnt<Store>
where
    Store: 'db,
{
    type Get = Rc<Store>;

    type Brw<'a> = &'a Store where Self :'a;

    fn from_store(s: Store) -> Self {
        ValCnt { data: Rc::new(s) }
    }

    fn brw(&self) -> Self::Brw<'_> {
        &self.data
    }
}

impl<'db, Store> StableImmutVal<'db, Store> for ValCnt<Store>
where
    Store: 'db,
{
    fn get(&self) -> Self::Get {
        self.data.clone()
    }
}

impl<'db, Store> UnStableImmutVal<'db, Store> for ValCnt<Store>
where
    Store: 'db,
{
    type GetAux = ();

    fn get(&self, _aux: Self::GetAux) -> Self::Get {
        self.data.clone()
    }
}

unsafe impl<'db, Store> super::Pullable<'db, Store> for ValCnt<Store>
where
    Store: 'db,
{
    type Own = Rc<Store>;

    fn pull(&mut self) -> Self::Own {
        // BUG: cost of incrementing reference count, and a leak
        //      consider MaybeUninit to avoid? Or an Option (no size cost)?
        self.data.clone()
    }

    fn place(&mut self, s: Store) {
        self.data = Rc::new(s);
    }
}
