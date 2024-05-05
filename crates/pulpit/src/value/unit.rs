//! # Empty immutable value
//! A zero-overhead placeholder for columns with no immutable parts 
//! - We avoid using a tuple based implementation, as it returns references, or index wrapped.
//! ```
//! assert_eq!(std::mem::size_of::<&()>, 8) // some cost!
//! assert_eq!(std::mem::size_of::<()>, 0)
//! ```
use crate::column;
struct Unit;

unsafe impl <'db> super::ImmutVal<'db, ()> for Unit {
    type Get = ();
    type Brw<'a> = () where Self: 'a;

    fn from_store(_s: ()) -> Self {
        Unit
    }
    
    fn brw<'a>(&'a self) -> Self::Brw<'a> {
        ()
    }

} 
unsafe impl <'db> super::StableImmutVal<'db, ()> for Unit {
    fn get(&self) -> () {
        ()
    }
}

unsafe impl <'db, StoreMut, Col: column::Column<'db, StoreMut, ()>> super::UnStableImmutVal<'db, StoreMut, (), Col> for Unit {
    fn get(&self, _ind: <Col as column::Column<'db, StoreMut, ()>>::Ind, _col: &Col) -> Self::Get {
        ()
    }
}

unsafe impl <'db> super::Pullable<'db, ()> for Unit {
    type Own = ();
    
    fn pull(&mut self) -> Self::Own {
        ()
    }
}