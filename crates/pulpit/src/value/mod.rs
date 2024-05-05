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
//! ## Mutable Side of [Columns](column::Column)
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

use crate::column;

mod unit;
mod valind;
mod valref;
mod valcpy;


/// Wraps an immutable value that is part of a column 
/// - Values can be safely shared for lifetime of the column
/// - Borrow of the column does not prevent mutable borrows of the mutable side 
///   of the column from occuring
pub unsafe trait ImmutVal<'db, Store> {
    /// The value to provide for get - share (valid for lifetime of database), but not a borrow. 
    /// (Close to zero-cost)
    type Get: 'db;

    /// The value to present for normal borrows
    /// (Zero-cost)
    type Brw<'a> where Self: 'a;

    fn from_store(s: Store) -> Self;
    fn brw<'a>(&'a self) -> Self::Brw<'a>;
}

/// Used when pointer stability is ensured.
/// - The value is not moved when changes occur to the containing column.
pub unsafe trait StableImmutVal<'db, Store>: ImmutVal<'db, Store> {
    fn get(&self) -> Self::Get;
}

/// Used when pointer stability is not ensured.
/// - Shared references need to re-index the column on access.
pub unsafe trait UnStableImmutVal<'db, StoreMut, Store, Col: column::Column<'db, StoreMut, Store>>: ImmutVal<'db, Store> {
    // TODO: determine if I need to have `col: Pin<&Col>`
    fn get(&self, ind: Col::Ind, col: &Col) -> Self::Get;
}

/// Used for values which cannot be updated, but can be deleted.
pub unsafe trait Pullable<'db, Store> : ImmutVal<'db, Store> {
    /// The value to take ownership when pulling
    type Own;

    fn pull(&mut self) -> Self::Own;
}