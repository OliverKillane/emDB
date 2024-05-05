//! # Indexed values for Unstable Columns
//! When a column does not provide pointer stability, we need to provide
//! references are indices into the column.
//!
//! From an index, the value can be borrowed. However the duration of the borrow
//! needs to be considered.
//! - We need to prevent mutation of the column that could cause values to be
//!   moved (e.g. to a larger allocation on insert)
//! - We need to ensure the borrows do not live longer than the column itself
//!
//! To achieve this requires 1 of two techniques:
//! 1. Include a lock in the column that can be safely acquired and released by
//!    the borrow
//! 2. Prevent the column from being shared between threads (using !Send, !Sync).
//!    Mark borrowed items in the column, and prevent release of resources until
//!    the borrows are gone (borrows wrapped by a structure that marks on drop)
//! 3. Just copy the dang values
//! 4. Pass a column reference when dereferencing, this reference prevents
//!    mutation (possibly affecting stability) of the column.
//!  
//! Here [`ValInd`] uses the final option, as it keeps the door open for
//! concurrency, without any compromises with regards to tracking references by
//! the column.
//!
//!

#[allow(dead_code)]
pub struct ValInd<Val> {
    data: Val,
}

/*

TODO: Decide if to keep this, very compilcated generic mess.

use crate::column;
use super::ImmutVal;

struct IndWrap<'db, StoreMut, Store, SubVal, Col>
where
    Col: column::Column<'db, StoreMut, Store, ImmVal = ValInd<Store>>,
    ValInd<Store> : ImmutVal<'db, Store>,
    // NOTE: In order for the `for<'a> Fn(..) -> &'a ..` to work, we need to use the lifetime
    //       for a reference in the `Fn`'s input. Hence `&'a Col` is just a convenient dummy here
{
    ind: Col::Ind,
    select: for<'a> fn(<ValInd<Store> as ImmutVal<'db, Store>>::Brw<'a>, &'a Col) -> &'a SubVal,
}

impl <'db, StoreMut, Store, SubVal, Col> IndWrap<'db, StoreMut, Store, SubVal, Col>
where
    Col: column::Column<'db, StoreMut, Store, ImmVal = ValInd<Store>>,
    ValInd<Store> : ImmutVal<'db, Store>,
{
    /// Take a reference to the column (prevent relocation) borrow using the
    /// indexing ref.
    fn deref<'a>(&'a self, col: &'a Col) -> &'a SubVal {
        let (imm, _) = col.brw(self.ind);
        (self.select)(imm, col)
    }
}

trait StoreType {
    type Tup;
}

macro_rules! tuple_impl {
    ($($t:ident),+ ) => {

        impl <$($t),+> StoreType for ValInd<($($t),+)> {
            type Tup = ($($t),+);
        }

        unsafe impl <'db, StoreMut, $($t),+ , Col: column::Column<'db, StoreMut, ($($t),+)>> ImmutVal<'db, ($($t),+)> for ValInd<($($t),+)>
        where
          $($t : 'db),+ ,
          ValInd<($($t),+)>: StoreType<Tup=($($t),+)>,
          Col::ImmVal: ImmutVal<'db, ValInd<($($t),+)>>
        {
            type Get = ($(IndWrap<'db, StoreMut, <Self as StoreType>::Tup, $t, Col>),+);
        }

        tuple_impl!{=> $($t),+}
    };
    (=> $miss:ident , $ignore:ident ) => {};
    (=> $miss:ident, $incl:ident , $($t:ident),+ ) => {
        tuple_impl!{ $incl , $($t),+ }
    };
}

tuple_impl!{A,B}

*/
