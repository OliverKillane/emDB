//! # references to stable values.
//! Return a reference bounded by the `'db'` lifetime parameter.
//! - Fast, but requires careful handling of lifetimes.
//! - Can bind the ['db] lifetime as specified in [`ImmutVal`] documentation

use super::{ImmutVal, StableImmutVal};
use std::cell::UnsafeCell;

pub struct ValRef<Val> {
    data: UnsafeCell<Val>,
}

/// ## Lifetime Trickery & Unsafety
/// We can set the lifetime for which the `get` should be valid using the `'db`
/// lifetime parameter.
///
/// As a result, it is possible to create a [`ImmutVal::Get`] that is valid longer
/// than the object itself correctly (e.g. copies, ref counting), or incorrectly
/// (using [`ValRef`] and returning a reference that outlasts the object).
///
/// We can see this in the following example
/// ```compile_fail,E0505
/// # use pulpit::value::{ImmutVal, StableImmutVal, ValRef};
/// struct UnitNoCopy;
/// type TestType = (i32, i32);
/// fn set_lifetime<'db, V: ImmutVal<'db, TestType> + StableImmutVal<'db, TestType>>(x: &'db UnitNoCopy, n: TestType) -> V::Get {
///     let mut z: (i32, V) = (3, V::from_store(n));
///     let x = &mut z.0;
///     let x_get_fail = z.1.get();
///     *x += 2;
///     let y = &mut z;
///     x_get_fail
/// }
///
/// fn lifetime_check() {
///     let out: &i32;
///     let dummy = UnitNoCopy; // fails on the drop
///     {
///         // we use dummy to set lifetime of the borrow.
///         // this is fundamentally unsafe (hence the unsafe trait), I should constrain to the tuple itself.
///         // without the dummy type to 'transfer' its lifetime over to the contained immutable value references, this fails.
///         
///         // let dummy = UnitNoCopy; // fails on the `out;` after the block - correct
///         let (x, y) = set_lifetime::<ValRef<(i32, i32)>>(&dummy, (1,2));
///         out = x;
///     }
///     // out is a dangling reference here, because out uses the lifetime of `dummy`, not `y`.
///     out;
///     drop(dummy);
///     // out is invalidated, and the next use fails
///     let y = out.clone();
/// }
/// ```
///
/// TODO: I want to find a nicer way here, to express `'db` as the lifetime of the [`ValRef`].
/// Somehow bind the lifetime of self and this together? `Self: 'db`?
impl<'db, Store> ImmutVal<'db, Store> for ValRef<Store>
where
    Store: 'db,
{
    type Get = &'db Store;
    type Brw<'a> = &'a Store where Self: 'a;

    #[inline(always)]
    fn from_store(s: Store) -> Self {
        ValRef {
            data: UnsafeCell::new(s),
        }
    }

    #[inline(always)]
    fn brw(&self) -> Self::Brw<'_> {
        unsafe { &*self.data.get() }
    }
}

impl<'db, Store> StableImmutVal<'db, Store> for ValRef<Store>
where
    Store: 'db,
{
    #[inline(always)]
    fn get(&self) -> Self::Get {
        unsafe { &*self.data.get() }
    }
}
