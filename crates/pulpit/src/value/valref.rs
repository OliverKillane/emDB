//! # references to stable values.
//! Return a reference bounded by the `'db'` lifetime parameter.
//! - Fast, but requires careful handling of lifetimes.
//! - Can bind the ['db] lifetime as specified in [`ImmutVal`] documentation

use super::{ImmutVal, StableImmutVal};
use std::cell::UnsafeCell;

pub struct ValRef<Val> {
    data: UnsafeCell<Val>,
}

macro_rules! tuple_impl {
    ($($t:ident),+ ) => {
        impl <'db, $($t),+> ImmutVal<'db, ($($t),+)> for ValRef<($($t),+)>
        where
          $($t : 'db),+ ,
         {
            type Get = ($(&'db $t),+);
            type Brw<'a> = ($(&'a $t),+) where Self: 'a;

            #[inline(always)]
            fn from_store(s: ($($t),+)) -> Self {
                ValRef {
                    data: UnsafeCell::new(s),
                }
            }

            #[inline(always)]
            fn brw(&self) -> Self::Brw<'_> {
                #[allow(non_snake_case)]
                let ($($t),+) = unsafe { &*self.data.get() };
                ($($t),+)
            }
        }

        impl <'db, $($t),+> StableImmutVal<'db, ($($t),+)> for ValRef<($($t),+)>
        where
          $($t : 'db),+
         {
            #[inline(always)]
            fn get(&self) -> Self::Get {
                #[allow(non_snake_case)]
                let ($($t),+) = unsafe { &*self.data.get() };
                ($($t),+)
            }
        }

        tuple_impl!{=> $($t),+}

    };

    // Only one value left, dont implement as `(T)` is actually `T` which will shadow all `(...)`
    (=> $miss:ident , $ignore:ident ) => {};
    (=> $miss:ident, $incl:ident , $($t:ident),+ ) => {
        tuple_impl!{ $incl , $($t),+ }
    };
}

tuple_impl! {A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z}
