//! # Get copies of immutable values.
//! Used for small types for which a copy is better than returning a reference to dereference.
//!
//! This also means that pointer stability can be ignored (no references, just copies)

use super::{ImmutVal, Pullable, StableImmutVal, UnStableImmutVal};

pub struct ValCpy<Val> {
    data: Val,
}

macro_rules! tuple_impl {
    ($($t:ident),+ ) => {
        impl <'db, $($t),+> ImmutVal<'db, ($($t),+)> for ValCpy<($($t),+)>
        where
          $($t : 'db),+ ,
          $($t : Clone),+
        {
            type Get = ($($t),+);
            type Brw<'a> = ($(&'a $t),+) where Self: 'a;

            #[inline(always)]
            fn from_store(s: ($($t),+)) -> Self {
                ValCpy {
                    data: s,
                }
            }

            #[inline(always)]
            fn brw(&self) -> Self::Brw<'_> {
                #[allow(non_snake_case)]
                let ($($t),+) = &self.data;
                ($($t),+)
            }
        }

        impl <'db, $($t),+> StableImmutVal<'db, ($($t),+)> for ValCpy<($($t),+)>
        where
          $($t : 'db),+ ,
          $($t : Clone),+
        {
            #[inline(always)]
            fn get(&self) -> Self::Get {
                self.data.clone()
            }
        }

        impl <'db, $($t),+ > UnStableImmutVal<'db, ($($t),+)> for ValCpy<($($t),+)>
        where
            $($t : 'db),+ ,
            $($t : Clone),+
        {
            type GetAux = ();

            #[inline(always)]
            fn get(&self, _aux: Self::GetAux) -> Self::Get {
                self.data.clone()
            }
        }

        unsafe impl <'db,  $($t),+ > Pullable<'db, ( $($t),+ )> for ValCpy<( $($t),+ )>
        where
            $($t : 'db),+ ,
            $($t : Clone),+
        {
            type Own = ($($t),+);

            #[inline(always)]
            fn pull(&mut self) -> Self::Own {
                self.data.clone()
            }

            fn place(&mut self, s: Self::Own) {
                self.data = s;
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
