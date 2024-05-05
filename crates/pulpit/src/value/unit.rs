//! # Empty immutable value
//! A zero-overhead placeholder for columns with no immutable parts
//! - We avoid using a tuple based implementation, as it returns references, or index wrapped.
//! ```
//! assert_eq!(std::mem::size_of::<&()>(), 8); // some cost!
//! assert_eq!(std::mem::size_of::<()>(), 0);
//! ```

pub struct Unit;

impl<'db> super::ImmutVal<'db, ()> for Unit {
    type Get = ();
    type Brw<'a> = () where Self: 'a;

    #[inline(always)]
    fn from_store(_s: ()) -> Self {
        Unit
    }

    #[inline(always)]
    fn brw(&self) -> Self::Brw<'_> {}
}
impl<'db> super::StableImmutVal<'db, ()> for Unit {
    #[inline(always)]
    fn get(&self) {}
}

impl<'db> super::UnStableImmutVal<'db, ()> for Unit {
    type GetAux = ();

    #[inline(always)]
    fn get(&self, _aux: Self::GetAux) -> Self::Get {}
}

unsafe impl<'db> super::Pullable<'db, ()> for Unit {
    type Own = ();

    #[inline(always)]
    fn pull(&mut self) -> Self::Own {}

    fn place(&mut self, _s: ()) {}
}
