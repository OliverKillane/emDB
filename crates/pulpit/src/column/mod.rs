//! # Table Columns
//! Traits and implementations for the basic column structures accessed through.

use crate::value;

mod colvec;
mod colmap;
mod colblok;

pub enum ColumnError {
    AllocationFailure,
}

/// A column is an indexable data structure containing a set of values.
pub unsafe trait Column<'db, StoreMut, ImmStore> {
    type Ind: Copy;
    type ImmVal: value::ImmutVal<'db, ImmStore>;

    /// Get the value from the index, if immutable a form of reference is 
    /// returned, otherwise a copy.
    /// - `INV`: the index is valid
    fn get(&self, ind: Self::Ind) -> (<Self::ImmVal as value::ImmutVal<'db, ImmStore>>::Get, StoreMut);
    
    /// Immutably borrow the values from an entry.
    /// - `INV`: the index is valid
    fn brw<'a>(&'a self, ind: Self::Ind) -> (<Self::ImmVal as value::ImmutVal<'db, ImmStore>>::Brw<'a>, &'a StoreMut) where 'db: 'a;

    /// Mutably borrow the values from an entry.
    /// - `INV`: the index is valid
    fn brw_mut<'a>(&'a mut self, ind: Self::Ind) -> &'a mut StoreMut;
    
    /// Add a new value to the column:
    /// - `INV`: the index the value is placed at is the next highest index.
    fn put_new(&mut self, x: (StoreMut, ImmStore)) -> Result<(), ColumnError>;
}

/// A column that supports deletions
pub unsafe trait ColumnDel<'db, StoreMut, ImmStore> : Column<'db, StoreMut, ImmStore> where Self::ImmVal: value::Pullable<'db, ImmStore>{
    /// Place a value in a row that has been deleted.
    /// - `INV`: the index is valid
    fn place(&mut self, ind: Self::Ind, x: (StoreMut, ImmStore));

    /// Delete a row, and move its contents out.
    /// - `INV`: the index is valid
    /// - `INV`: the next access to this index is [ColumnDel::place]
    fn pull(&mut self, ind: Self::Ind) -> Result<<Self::ImmVal as value::Pullable<'db, ImmStore>>::Own, ColumnError>;
}

