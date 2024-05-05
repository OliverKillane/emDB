//! # Table Columns
//! Traits and implementations for the basic column structures accessed through.

use crate::value;

mod colblok;
mod colbuff;
mod colmap;
mod colvec;

pub enum ColumnError {
    AllocationFailure,
}

pub type ColInd = usize;

/// A column is an indexable data structure containing a set of values.
///
/// # Safety
/// When implementing indicied ([`ColInd`]) are assumed to be correct (checked by the table's index).
pub unsafe trait Column<'db, StoreMut, Store> {
    type ImmVal: value::ImmutVal<'db, Store>;

    /// Get the value from the index, if immutable a form of reference is
    /// returned, otherwise a copy.
    /// - `INV`: the index is valid
    fn get(&self, ind: ColInd) -> (<Self::ImmVal as value::ImmutVal<'db, Store>>::Get, StoreMut);

    /// Immutably borrow the values from an entry.
    /// - `INV`: the index is valid
    fn brw<'a>(
        &'a self,
        ind: ColInd,
    ) -> (
        <Self::ImmVal as value::ImmutVal<'db, Store>>::Brw<'a>,
        &'a StoreMut,
    )
    where
        'db: 'a;

    /// Mutably borrow the values from an entry.
    /// - `INV`: the index is valid
    fn brw_mut(&mut self, ind: ColInd) -> &mut StoreMut;

    /// Add a new value to the column:
    /// - `INV`: the index the value is placed at is the next highest index.
    fn put_new(&mut self, x: (StoreMut, Store)) -> Result<(), ColumnError>;
}

/// A column that supports deletions.
///
/// # Safety
/// When implemented the following assumptions can be made:
/// - a [`ColumnDel::pull`] only occurs on a valid index that has data contained (by [`ColumnDel::place`] or [`Column::put_new`])
/// - a [`ColumnDel::place`] only occurs on a valid index that has been deleted (by [`ColumnDel::pull`])
pub unsafe trait ColumnDel<'db, StoreMut, Store>: Column<'db, StoreMut, Store>
where
    Self::ImmVal: value::Pullable<'db, Store>,
{
    /// Place a value in a row that has been deleted.
    /// - `INV`: the index is valid
    fn place(&mut self, ind: ColInd, x: (StoreMut, Store));

    /// Delete a row, and move its contents out.
    /// - `INV`: the index is valid
    /// - `INV`: the next access to this index is [ColumnDel::place]
    fn pull(
        &mut self,
        ind: ColInd,
    ) -> Result<<Self::ImmVal as value::Pullable<'db, Store>>::Own, ColumnError>;
}
