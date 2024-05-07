//! # Table Columns
//! Traits and implementations for the basic column structures accessed through.
//!
//! ## Immutability Advantage
//! This column interface explicitly supports demarking parts of each row as
//! immutable to allow for performance improvements by avoiding copying data.
//!
//! - All data is moved on insert (move strictly cheaper than copy)
//! - All data can be borrowed (avoid copying for short borrow)
//! - Immutable data gotten with copy or cheaper (from borrow, to reindex, to copy)
//! - Mutable data gotten must be copied (table can be mutated after)
//!
//! For tables supporting [`ColumnWindowPull`]
//! - All data can be pulled (mutable by move, immutable by move or cheaper - e.g. cloning [`Rc`](std::rc::Rc))
//!
//! This advantage is significant when compared to conventional tables in embedded
//! database systems which require:
//! 1. Copy the value over to database (cannot take ownership of heap memory)
//! 2. Do database things in the database, cannot guarentee immutability while
//!    supporting ad-hoc queries, so some copies occur during query execution
//! 3. Copy the result back over to the user (user cannot safely reference memory
//!    inside the database)
//!
//! ## Referencing Immutable Data
//! In order to safely return references to immutable data while allowing further
//! referencing and mutation of the mutable data, we need the following:
//! 1. Guarentee the referenced data is not mutated (simple to verify)
//! 2. Guarentee the referenced data is not moved within the column (e.g. like a [`Vec`] reallocating on insert)
//!    (use different kinds of references)
//! 3. Limit the reference to the lifetime that the column is not moved (difficult)
//!
//! In order to achieve (3.) we need to attah the lifetime of the column to
//! returned references.
//!
//! ### Using the [interior mutability pattern](https://doc.rust-lang.org/reference/interior-mutability.html)
//! As all references are reads, this is just the lifetime of `&self` in a trait method.
//!   
//! However interior mutability removes some of the safety in the user interface,
//! we need to check mutations, but can no longer rely on the borrow checker to do
//! so.
//!
//! Hence use of [`std::cell::RefCell`] or locking the column with [`std::sync::RwLock`].
//!
//! ### Using an access Token
//! By using the lifetime of the borrowed token as a lifetime parameter to the Column
//! to use in qualifying references, we can control the lifetime of immutable references.
//!
//! However, we need to ensure the token does not live longer than the column,
//! otherwise we can get dangling references.
//!
//! ```no_run
//! struct Token; // Zero-Size Token

//! struct Data<'imm, ImmData, MutData> {
//!     imm_data: ImmData,
//!     mut_data: MutData,
//!     tk: &'imm Token // We could just steal the lifetime with a phantomdata
//! }
//!
//! impl <'imm, ImmData, MutData> Data<'imm, ImmData, MutData> {
//!     fn get_imm(&self) -> &'imm ImmData {
//!         unsafe {
//!             std::mem::transmute(&self.imm_data)
//!         }
//!     }
//!
//!     fn get_mut(&mut self) -> &mut MutData {
//!         &mut self.mut_data
//!     }
//! }
//!
//! fn test() {
//!     let tk = Token; // Token lives longer than the data
//!     let imm_ref;
//!     {
//!         let mut data = Data {
//!             imm_data: 3,
//!             mut_data: 4,
//!             tk: &tk
//!         };
//!         // Get immutable
//!         let x1 = data.get_imm();
//!         // mutable borrow or mutable field does not conflict (GOOD)
//!         let y1 = data.get_mut();
//!         // immutable borrow still present without conflict
//!         let z1 = *x1;
//!         imm_ref = x1;
//!     }
//!     // `tk` lives to here, but `data` did not
//!     let z2 = *imm_ref; // ERROR! dereferencing dangling reference
//! }
//! ```
//!
//! ### Using a Window
//! In order to solve this issue with tokens outliving values, we can instead
//! flip the roles. Place the data in the token (as `Column`), and allow only one `Window`
//! into the `Column` (enforced using the borrow checker and a `&mut` of the `Column`)
//!
//! This allows for the compiler to check borrows from the safe interface (no
//! runtime checks as with interior mutability), while preventing any dangling
//! references (immutable borrows properly qualified).
//!
//! This implementation is chosen in the form of [`ColumnWindow`], which is a window into its [`ColumnWindow::Col`].
//!
//! ## Immutable Value Storage
//! ### Pullability
//! The delete operation on tables is expressed through [`ColumnWindowPull`], here pulling the value
//! (ideally a move) from the table for the user.
//!
//! This affects references to values, if a value is pulled from a column,
//! references to it may be invalidated. Solutions include:
//!
//! 1. Keeping values alive until the column is destroyed, in a stable allocation (e.g. a box).
//! 2. Using reference counted values, stored separately.
//! 3. Rather than getting values, just re-index and borrow later - it is immutable data after all, copy on get.
//!
//! ### Pointer Stability
//! Columns internally may want to reallocate where data is placed, which will
//! invalidate references to data in the table.
//!
//! To prevent this requires placing the data in some separate stable allocation
//! that can be referenced, or copying.
//!

mod colblok;
mod colbuff;
mod colmap;
mod colvec;


pub type ColInd = usize;

/// An indexable (with [`ColInd`]) data structure, that is windowable using [`Column::Window`].
pub trait Column<ImmStore, MutStore> {
    type Window<'imm>: ColumnWindow<'imm, ImmStore, MutStore, Col = Self>
    where
        ImmStore: 'imm,
        MutStore: 'imm;
    type InitData;

    fn new(init: Self::InitData) -> Self;
}

/// A view into an indexable data structure representing a column.
/// - Is indexed by [`ColInd`], with no bounds checks.
/// - Can store mutable and immutable data together safely.
/// - Provides a window to access data, immutable values can be borrowed for the
///   lifetime of the reference contained in the window.
pub trait ColumnWindow<'imm, ImmStore, MutStore> {
    /// Getting the immutable value for the lifetime of the [`ColumnWindow`]
    /// - Does not conflict with concurrent [`ColumnWindow::brw`], [`ColumnWindow::brw_mut`]
    ///   or any [`ColumnWindowPull`] operations.
    type GetVal<'brw>;

    /// The type of the data structure that owns the data accessed through the [`ColumnWindow`]
    /// - A [`ColumnWindow`] contains a mutable reference to this owner.
    type Col: Column<ImmStore, MutStore, Window<'imm> = Self>
    where
        ImmStore: 'imm,
        MutStore: 'imm;

    /// Create a new view, usable for `'imm` from a column.
    fn new_view(col: &'imm mut Self::Col) -> Self;

    /// Get a value from an index in the column, which should be accessible as a valid [`ColumnWindow::GetVal`]
    /// for `'imm`.
    /// - Not zero cost, but at least as cheap as [`Clone`]
    /// - Resulting [`ColumnWindow::GetVal`] can be held without blocking concurrent operations.
    ///
    /// # Safety
    /// - No bounds checks applied
    /// - index assumed to be in valid state
    unsafe fn get<'brw>(&'brw self, ind: ColInd) -> (Self::GetVal<'imm>, MutStore);

    /// Borrow a value from an index in the column for a smaller lifetime
    /// - Zero cost, a normal reference.
    ///
    /// # Safety
    /// - No bounds checks applied
    /// - index assumed to be in valid state
    unsafe fn brw(&self, ind: ColInd) -> (&ImmStore, &MutStore);

    /// Mutably borrow the mutable part of an index in the column.
    ///
    /// # Safety
    /// - No bounds checks applied
    /// - index assumed to be in valid state
    unsafe fn brw_mut(&mut self, ind: ColInd) -> &mut MutStore;

    /// Add a new value to the column at the next index.
    fn put_new(&mut self, x: (ImmStore, MutStore));
}

/// A view into a column that supports values being pulled/deleted from the column.
///
/// ## New States
/// An index can now be: `FULL` (has a value) or `PULLED` (index was removed)
pub trait ColumnWindowPull<'imm, ImmStore, MutStore>:
    ColumnWindow<'imm, ImmStore, MutStore>
{
    /// A value that can be pulled from the column, containing the original `ImmStore`
    /// data, potentially represented differently.
    type PullVal;

    /// Pull a value from an index. The index is in an `INVALID` state after
    /// this operation.
    ///
    /// # Safety
    /// - No bounds checks
    unsafe fn pull(&mut self, ind: ColInd) -> (Self::PullVal, MutStore);

    /// Place a value in an index that is in a `PULLED` state.
    ///
    /// # Safety
    /// - No bounds checks
    unsafe fn place(&mut self, ind: ColInd, x: (ImmStore, MutStore));
}
