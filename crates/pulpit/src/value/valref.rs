use std::{cell::UnsafeCell};
use super::{ImmutVal,StableImmutVal};
struct ValRef<Val> {
    data: UnsafeCell<Val>,
}

macro_rules! valref_tuple_impl {
    ($($t:ident),+ ) => {
        unsafe impl <'db, $($t),+> ImmutVal<'db, ($($t),+)> for ValRef<($($t),+)>
        where 
          $($t : 'db),+
         {
            type Get = ($(&'db $t),+);
            type Brw<'a> = ($(&'a $t),+) where Self: 'a;

            fn from_store(s: ($($t),+)) -> Self {
                ValRef {
                    data: UnsafeCell::new(s),
                }
            }

            fn brw<'a>(&'a self) -> Self::Brw<'a> {
                #[allow(non_snake_case)]
                let ($($t),+) = unsafe { &*self.data.get() };
                ($($t),+)
            }
        }

        unsafe impl <'db, $($t),+> StableImmutVal<'db, ($($t),+)> for ValRef<($($t),+)>
        where 
          $($t : 'db),+
         {
            fn get(&self) -> Self::Get {
                #[allow(non_snake_case)]
                let ($($t),+) = unsafe { &*self.data.get() };
                ($($t),+)
            }
        }

        valref_tuple_impl!{=> $($t),+}

    };

    // Only one value left, dont implement as `(T)` is actually `T` which will shadow all `(...)`
    (=> $miss:ident , $ignore:ident ) => {};
    (=> $miss:ident, $incl:ident , $($t:ident),+ ) => {
        valref_tuple_impl!{ $incl , $($t),+ } 
    };
}

valref_tuple_impl!{A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z}
