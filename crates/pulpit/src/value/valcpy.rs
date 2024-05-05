use super::{ImmutVal,StableImmutVal, UnStableImmutVal};
use crate::column;
struct ValCpy<Val> {
    data: Val,
}

macro_rules! valref_tuple_impl {
    ($($t:ident),+ ) => {
        unsafe impl <'db, $($t),+> ImmutVal<'db, ($($t),+)> for ValCpy<($($t),+)>
        where 
          $($t : 'db),+ ,
          $($t : Copy),+
        {
            type Get = ($($t),+);
            type Brw<'a> = ($(&'a $t),+) where Self: 'a;

            fn from_store(s: ($($t),+)) -> Self {
                ValCpy {
                    data: s,
                }
            }

            fn brw<'a>(&'a self) -> Self::Brw<'a> {
                #[allow(non_snake_case)]
                let ($($t),+) = &self.data;
                ($($t),+)
            }
        }

        unsafe impl <'db, $($t),+> StableImmutVal<'db, ($($t),+)> for ValCpy<($($t),+)>
        where 
          $($t : 'db),+ , 
          $($t : Copy),+
        {
            fn get(&self) -> Self::Get {
                self.data
            }
        }

        unsafe impl <'db, StoreMut, $($t),+ , Col: column::Column<'db, StoreMut, ($($t),+)>> UnStableImmutVal<'db, StoreMut, ($($t),+), Col> for ValCpy<($($t),+)> 
        where 
            $($t : 'db),+ , 
            $($t : Copy),+ 
        {
            fn get(&self, _ind: Col::Ind, _col: &Col) -> Self::Get {
                self.data
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
