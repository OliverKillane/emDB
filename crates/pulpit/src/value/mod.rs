//! # Wrapping immutable values
//! Managing values that cannot be updated to improve performance by avoiding
//! copies wherever possible.
//!
//! For small [`Copy`] items (i.e. less than 8 bytes), the value is copied, but
//! for larger types (e.g. strings) we want to avoid the cost of [`Clone`], but
//! still provide access.
//!
//! | Stability | Pullable   | How to Share                      |
//! |-----------|------------|-----------------------------------|
//! | Yes       | Yes        | Indirection with ref counting     |
//! | Yes       | No         | Reference with container lifetime |
//! | No        | Yes        | indirection with ref counting     |
//! | No        | No         | re-index column                   |
//!
//! For smaller types, copying is also fine.
//!
//! ## Mutable Side of [Columns](crate::column::Column)
//! Trickier to avoid copies, the main case being columns that *could* be mutated,
//! but some rows never are, so accesses to said rows should be references rather
//! than copies.  
//!
//! The only way to do this effectively is to clone on mutation, and replace.
//! ```ignore
//! // mutating a
//! let a_ref: &mut A = ...;
//! *new_a_place = a_ref.clone();
//! (mutation)(new_a_place);
//! // do some cleanup
//! ```
//!
//! The efficient way to do this is to use multiple generations of a column
//! - indexes can select generation
//! - updates increment generation
//! - store a new column for each generation
//!
//! ```ignore
//! columns: Vec<ColumnKind> = vec![ ... ]
//! // each index is a different generation
//! // indices are (generation, index)
//! // mutating a row -> increase current index generation
//! // getting a row -> return index with generation
//! ```
//!
//! But this brings further issues such as:
//! - Mapping indexes efficiently:
//!    | Case                     | Optimal                                            |
//!    |--------------------------|----------------------------------------------------|
//!    | All rows evenly mutated  | On new generation, allocate space for whole column |
//!    | All mutations on one row | Only allocate for that row, ignore the rest        |
//! - Out of place performance (e.g. if column is just a pointer to allocation, create
//!   new for each mutation), more copies occur & the columnar access pattern is lost.
//! Hence we do not specialise for mutable values.
//!
//! ## Complexity
//! Why not just [`std::rc::Rc`] eveything? Because it is not zero cost.
//!
//! ```
//! // reference counted values require 8 bytes on the stack for the pointer
//! assert_eq!(std::mem::size_of::<std::rc::Rc<usize>>(), 8);
//! // and requires an additional control block on the heap!
//! ```
//!
//! ## Enforcing Stability Constraints
//! This is done two ways:
//! 1. We split the stable and unstable pointer cases to separate traits, so
//!    columns can parameterise themselves by values according to their own
//!    element pointer sability
//! 2. We use `Pin`
//!

mod unit;
pub use unit::*;
mod valind;
pub use valind::*;
mod valref;
pub use valref::*;
mod valcpy;
pub use valcpy::*;
mod valbox;
pub use valbox::*;

/// Wraps an immutable value that is part of a column
/// - Values can be safely shared for lifetime of the column
/// - Borrow of the column does not prevent mutable borrows of the mutable side
///   of the column from occuring
///
/// With some lifetime trickery, we can convince rust that [`ImmutVal::Get`]ting
/// a value does not interact with borrows from the value.
///
/// ```compile_fail,E0502
/// # use pulpit::value::{ImmutVal, StableImmutVal};
/// # fn lifetime_check<'db, I: ImmutVal<'db, (i32, i32)> + StableImmutVal<'db, (i32, i32)>>() {
/// // representing a column with mutable and immutable parts
/// let mut z = (3, I::from_store((1,2)));
///    
/// // a normal borrow, so cannot also immutably borrow
/// let x_brw = z.1.brw();
/// let j = &mut z;
/// # }
/// ```
///
/// We can fix this by using [`StableImmutVal::get`] to get the stable &
/// immutable reference. We have ensured that `z.1` cannot be mutated.
///
/// ```
/// # use pulpit::value::{ImmutVal, StableImmutVal};
/// # fn lifetime_check<'db, I: ImmutVal<'db, (i32, i32)> + StableImmutVal<'db, (i32, i32)>>() {
/// // representing a column with mutable and immutable parts
/// let mut z = (3, I::from_store((1,2)));
///
/// // a normal borrow, so cannot also immutably borrow
/// let x_get = z.1.get();
/// let j = &mut z;
/// let x_get_move = x_get;
/// # }
/// ```
///
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
///         // let dummy = UnitNoCopy; // fails on the `out;` after the block
///         let (x, y) = set_lifetime::<ValRef<(i32, i32)>>(&dummy, (1,2));
///         out = x;
///     }
///     out; // causes compile failure
///     drop(dummy);
///     let y = out.clone();
/// }
/// ```
pub trait ImmutVal<'db, Store> {
    /// The value to provide for get - share (valid for lifetime of database), but not a borrow.
    /// (less or equal cost to [Clone])
    type Get: 'db;

    /// The value to present for normal borrows
    /// (Zero-cost)
    type Brw<'a>
    where
        Self: 'a;

    fn from_store(s: Store) -> Self;
    fn brw(&self) -> Self::Brw<'_>;
}

/// Used when pointer stability is ensured.
/// - The value is not moved when changes occur to the containing column.
pub trait StableImmutVal<'db, Store>: ImmutVal<'db, Store> {
    fn get(&self) -> Self::Get;
}

/// Used when pointer stability is not ensured.
/// - Shared references need to re-index the column on access.
pub trait UnStableImmutVal<'db, Store>: ImmutVal<'db, Store> {
    /// Auxiliary info needed for access to an unstable value (e.g. the column & index)
    type GetAux;

    // TODO: determine if I need to have `col: Pin<&Col>`
    fn get(&self, aux: Self::GetAux) -> Self::Get;
}

/// Used for values which cannot be updated, but can be deleted.
/// # Safety
/// When implemented the following assumptions can be made:
/// - a [`Pullable::pull`] only occurs on a valid index that has data contained (by [`Pullable::place`] or [`ImmutVal::from_store`])
/// - a [`Pullable::place`] only occurs on a valid index that has been deleted (by [`Pullable::pull`])
pub unsafe trait Pullable<'db, Store>: ImmutVal<'db, Store> {
    /// The value to take ownership when pulling
    type Own;

    /// `Safety INV`: The value is left in a state that is valid for
    ///               [`Pullable::place`] to be called.
    fn pull(&mut self) -> Self::Own;

    fn place(&mut self, s: Store);
}
