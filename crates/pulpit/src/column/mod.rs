//! # Primary and Associated Index Columns
//! Each pulpit table is composed of a primary column (accessed through user-visible
//! [keys](Keyable::Key)) and associated columns (accessed without bounds
//! checks through raw column indexes).
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
//! For tables supporting [`PrimaryWindowPull`] or [`AssocWindowPull`], the immutable data is retained
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
//! In order to achieve (3.) we need to attach the lifetime of the column to
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
//! This implementation is chosen in the form of [`Column::WindowKind`], which is a
//! single mutable borrow of the column.
//!
//! ## Immutable Value Storage
//! ### Pullability
//! The delete operation on tables is expressed through [`PrimaryWindowPull`]/[`AssocWindowPull`], here pulling the value
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
//! ## Why not separate indexes?
//! I originally considered having the index entirely separate to the data
//! storage, however as demonstrated in the `col_vs_tup` benchmark, the cost of
//! separate inserts (required for an index that need to keep generations) is high.
//! - Allows for other optimisations, such as in [`PrimaryRetain`]'s reuse of space for
//!   data, and for the mutable data for generation & free slot storage.

use std::{hash::Hash, marker::PhantomData, mem::transmute};

mod assoc_blocks;
pub use assoc_blocks::*;
mod assoc_vec;
pub use assoc_vec::*;
mod primary_gen_arena;
pub use primary_gen_arena::*;
mod primary_no_pull;
pub use primary_no_pull::*;
mod primary_pull;
pub use primary_pull::*;
mod primary_retain;
pub use primary_retain::*;
mod primary_thunderdome;
pub use primary_thunderdome::*;

/// A single window type holding a mutable references through which windows for
/// columns and primary indexes can be generated.
pub struct Window<'imm, Data> {
    inner: &'imm mut Data,
}

/// The trait for describing column construction and windowing.
pub trait Column {
    type WindowKind<'imm>
    where
        Self: 'imm;
    fn new(size_hint: usize) -> Self;
    fn window(&mut self) -> Self::WindowKind<'_>;
}

/// In order to get the Key (without needing the `'imm` lifetime parameter) it is
/// kept separate from the window, referenced through the column in the window.
pub trait Keyable {
    type Key: Copy + Eq;
}

/// The raw column index type (used for unchecked indexes)
pub type UnsafeIndex = usize;

#[derive(Clone)]
pub struct Data<ImmData, MutData> {
    pub imm_data: ImmData,
    pub mut_data: MutData,
}

impl<ImmData, MutData> Data<ImmData, MutData> {
    pub fn convert_imm<ImmDataProcessed>(
        self,
        trans: impl Fn(ImmData) -> ImmDataProcessed,
    ) -> Data<ImmDataProcessed, MutData> {
        let Self { imm_data, mut_data } = self;
        Data {
            imm_data: trans(imm_data),
            mut_data,
        }
    }
}

pub struct Entry<ImmData, MutData> {
    pub index: UnsafeIndex,
    pub data: Data<ImmData, MutData>,
}

pub type Access<Imm, Mut> = Result<Entry<Imm, Mut>, KeyError>;

pub enum InsertAction {
    Place(UnsafeIndex),
    Append,
}

/// For safe access to a [`PrimaryWindow`] with an incorrect index.
#[derive(Debug)]
pub struct KeyError;

/// A view into a primary index (bounds checked, and produced [`UnsafeIndex`]es
/// for access to associated columns).
pub trait PrimaryWindow<'imm, ImmData, MutData> {
    /// Getting the immutable value for the lifetime of the [`PrimaryWindow`]
    /// - Does not conflict with concurrent [`PrimaryWindow::brw`], [`PrimaryWindow::brw_mut`]
    ///   or any [`PrimaryWindowPull`] operations.
    type ImmGet: 'imm;

    /// The key type of backing column, used to get the type needed for key (which
    /// does not need the `'imm` lifetime parameter)
    type Col: Keyable + Column;

    fn get(&self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmGet, MutData>;
    fn brw(&self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &MutData>;
    fn brw_mut(&mut self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &mut MutData>;

    /// For testing include a conversion for the immutable value
    fn conv_get(get: Self::ImmGet) -> ImmData;

    fn scan<'brw>(&'brw self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + 'brw;
    fn count(&self) -> usize;
}

pub trait PrimaryWindowApp<'imm, ImmData, MutData>: PrimaryWindow<'imm, ImmData, MutData> {
    fn append(&mut self, val: Data<ImmData, MutData>) -> <Self::Col as Keyable>::Key;

    /// To allow for transactions to remove data from the table
    ///
    /// # Safety
    /// - All [`PrimaryWindow::get`] values must not be accessed from this call,
    ///   to when they are dropped.
    unsafe fn unppend(&mut self);
}

pub trait PrimaryWindowPull<'imm, ImmData, MutData>: PrimaryWindow<'imm, ImmData, MutData> {
    /// The immutable data that can be pulled from the table. This is separate from
    /// [`PrimaryWindow::ImmGet`]. Allows for deletions that take ownership of contained
    /// data.
    type ImmPull: 'imm;

    /// n insert must track if old [`UnsafeIndex`] is to be overwritten in
    /// [`AssocWindowPull`] or an append is required.
    fn insert(
        &mut self,
        val: Data<ImmData, MutData>,
    ) -> (<Self::Col as Keyable>::Key, InsertAction);

    /// Pull data from a column (removes it from the column)
    /// For tables implementing [`PrimaryWindowHide`], this can include hidden
    /// values.
    fn pull(&mut self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmPull, MutData>;

    /// For testing include a conversion for the immutable value pulled
    fn conv_pull(pull: Self::ImmPull) -> ImmData;
}

/// Hides a given key temporarily, until revealed or removed.
/// - Allows for 'deletions' that are not actually enforced until commit.
/// - Allows the deletion from other associated columns to be postponed till the
///   end of a transaction.
pub trait PrimaryWindowHide<'imm, ImmData, MutData>:
    PrimaryWindowPull<'imm, ImmData, MutData>
{
    /// Hide a value from get and brw access.
    /// - Can be pulled from the table, or releaved (back to normal row)
    /// - Cannot be hidden twice
    fn hide(&mut self, key: <Self::Col as Keyable>::Key) -> Result<(), KeyError>;

    /// Un-Hide a value to return it to its normal state
    /// - Panics be called on a currently available row
    fn reveal(&mut self, key: <Self::Col as Keyable>::Key) -> Result<(), KeyError>;
}

pub trait AssocWindow<'imm, ImmData, MutData> {
    type ImmGet: 'imm;

    /// Get the value of the given [`UnsafeIndex`], that lives as long as the window
    /// - Not zero cost, but at least as cheap as [`Clone`]
    /// - Resulting [`AssocWindow::ImmGet`] can be held without blocking concurrent operations.
    ///
    /// # Safety
    /// - No bounds checks applied
    /// - index assumed to be in valid state
    unsafe fn get(&self, ind: UnsafeIndex) -> Data<Self::ImmGet, MutData>;

    /// Borrow a value from an index in the column for a smaller lifetime
    /// - Zero cost, a normal reference.
    ///
    /// # Safety
    /// - No bounds checks applied
    /// - index assumed to be in valid state
    unsafe fn brw(&self, ind: UnsafeIndex) -> Data<&ImmData, &MutData>;

    /// Mutably borrow the mutable part of an index in the column.
    ///
    /// # Safety
    /// - No bounds checks applied
    /// - index assumed to be in valid state
    unsafe fn brw_mut(&mut self, ind: UnsafeIndex) -> Data<&ImmData, &mut MutData>;

    /// Append a value to the column that is at the new largest [`UnsafeIndex`].
    fn append(&mut self, val: Data<ImmData, MutData>);

    /// To allow for transactions to remove data from the table
    ///
    /// # Safety
    /// - All [`AssocWindow::get`] values must not be accessed from this call,
    ///   to when they are dropped.
    unsafe fn unppend(&mut self);

    /// For testing include a conversion for the immutable value
    fn conv_get(get: Self::ImmGet) -> ImmData;
}

pub trait AssocWindowPull<'imm, ImmData, MutData>: AssocWindow<'imm, ImmData, MutData> {
    type ImmPull: 'imm;

    /// Pull a value from an index. The index is in an `INVALID` state after
    /// this operation.
    ///
    /// # Safety
    /// - No bounds checks
    unsafe fn pull(&mut self, ind: UnsafeIndex) -> Data<Self::ImmPull, MutData>;

    /// Place a value in an index that is in a `PULLED` state.
    ///
    /// # Safety
    /// - No bounds checks
    unsafe fn place(&mut self, ind: UnsafeIndex, val: Data<ImmData, MutData>);

    /// For testing include a conversion for the immutable value pulled
    fn conv_pull(pull: Self::ImmPull) -> ImmData;
}

/// A Simple Generational Index Key
pub struct GenKey<Store, GenCounter: Copy + Eq> {
    index: UnsafeIndex,
    generation: GenCounter,
    phantom: PhantomData<Store>,
}

impl<Store, GenCounter: Copy + Eq> PartialEq for GenKey<Store, GenCounter> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}
impl<Store, GenCounter: Copy + Eq> Eq for GenKey<Store, GenCounter> {}
impl<Store, GenCounter: Copy + Eq> Clone for GenKey<Store, GenCounter> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Store, GenCounter: Copy + Eq> Copy for GenKey<Store, GenCounter> {}
impl<Store, GenCounter: Copy + Eq + Hash> Hash for GenKey<Store, GenCounter> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}
mod utils {
    use std::mem::MaybeUninit;

    /// A sequence of allocated blocks providing stable pointers.
    pub struct Blocks<Value, const BLOCK_SIZE: usize> {
        count: usize,
        data: Vec<Box<[MaybeUninit<Value>; BLOCK_SIZE]>>,
    }

    impl<Value, const BLOCK_SIZE: usize> Drop for Blocks<Value, BLOCK_SIZE> {
        fn drop(&mut self) {
            for alive in 0..self.count {
                let (block, seq) = quotrem::<BLOCK_SIZE>(alive);
                unsafe {
                    self.data.get_unchecked_mut(block)[seq].assume_init_drop();
                }
            }
        }
    }

    impl<Value, const BLOCK_SIZE: usize> Blocks<Value, BLOCK_SIZE> {
        pub fn new(size_hint: usize) -> Self {
            Blocks {
                count: 0,
                data: Vec::with_capacity(size_hint / BLOCK_SIZE + 1),
            }
        }

        pub fn append(&mut self, val: Value) -> *mut Value {
            let (block, seq) = quotrem::<BLOCK_SIZE>(self.count);
            let data_ptr;
            unsafe {
                if seq == 0 {
                    self.data
                        .push(Box::new(MaybeUninit::uninit().assume_init()));
                }
                data_ptr = self.data.get_unchecked_mut(block)[seq].as_mut_ptr();
                data_ptr.write(val);
            }
            self.count += 1;
            data_ptr
        }

        /// Must not be used if references to the value still exist.
        pub unsafe fn unppend(&mut self) {
            let (block, seq) = quotrem::<BLOCK_SIZE>(self.count - 1);
            self.data.get_unchecked_mut(block)[seq].assume_init_drop();
            self.count -= 1;
        }

        pub unsafe fn get(&self, ind: usize) -> &Value {
            let (block, seq) = quotrem::<BLOCK_SIZE>(ind);
            self.data.get_unchecked(block)[seq].assume_init_ref()
        }

        pub unsafe fn get_mut(&mut self, ind: usize) -> &mut Value {
            let (block, seq) = quotrem::<BLOCK_SIZE>(ind);
            self.data.get_unchecked_mut(block)[seq].assume_init_mut()
        }
    }

    pub fn quotrem<const DIV: usize>(val: usize) -> (usize, usize) {
        (val / DIV, val % DIV)
    }
}

#[cfg(any(test, kani))]
mod verif {
    use super::*;
    use std::collections::HashMap;

    trait ReferenceMap<Key, Value> {
        fn with_capacity(size_hint: usize) -> Self;
        fn get(&self, key: &Key) -> Option<&Value>;
        fn insert(&mut self, key: Key, value: Value) -> Option<Value>;
        fn remove(&mut self, key: &Key) -> Option<Value>;
        fn len(&self) -> usize;
        fn get_next_key(&self) -> Option<Key>;
    }

    /// A very simple (and horribly inefficient) map, that is far faster to
    /// verify than the (efficient) HashMap.
    /// As verification of a [`primaryWindow`] requires tracking with a map,
    /// we need to use this.
    struct SimpleMap<Key, Value> {
        data: Vec<Option<(Key, Value)>>,
        count: usize,
    }

    impl<Key: Eq + Clone, Value: Clone> ReferenceMap<Key, Value> for SimpleMap<Key, Value> {
        fn with_capacity(size_hint: usize) -> Self {
            Self {
                data: Vec::with_capacity(size_hint),
                count: 0,
            }
        }

        fn get(&self, key: &Key) -> Option<&Value> {
            self.data.iter().find_map(|entry| {
                if let Some((k, v)) = entry {
                    if k == key {
                        Some(v)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        }

        fn insert(&mut self, key: Key, value: Value) -> Option<Value> {
            if let Some(v) = self.get(&key) {
                Some(v.clone())
            } else {
                self.data.push(Some((key, value)));
                self.count += 1;
                None
            }
        }

        fn remove(&mut self, key: &Key) -> Option<Value> {
            let val = self.data.iter_mut().find_map(|entry| {
                if let Some((k, _)) = entry {
                    if k == key {
                        Some(entry)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })?;

            val.take().map(|(_, v)| {
                self.count -= 1;
                v
            })
        }

        fn len(&self) -> usize {
            self.count
        }

        fn get_next_key(&self) -> Option<Key> {
            self.data.iter().find_map(|entry| {
                if let Some((k, _)) = entry {
                    Some(k.clone())
                } else {
                    None
                }
            })
        }
    }

    impl<Key: Eq + Hash + Clone, Value> ReferenceMap<Key, Value> for HashMap<Key, Value> {
        fn with_capacity(size_hint: usize) -> Self {
            HashMap::with_capacity(size_hint)
        }

        fn get(&self, key: &Key) -> Option<&Value> {
            self.get(key)
        }

        fn insert(&mut self, key: Key, value: Value) -> Option<Value> {
            self.insert(key, value)
        }

        fn remove(&mut self, key: &Key) -> Option<Value> {
            self.remove(key)
        }

        fn len(&self) -> usize {
            self.len()
        }

        fn get_next_key(&self) -> Option<Key> {
            self.keys().next().cloned()
        }
    }

    // A wrapper to check the correct
    struct CheckPrimary<'imm, ImmData, MutData, ColWindow, RefMap>
    where
        ColWindow: PrimaryWindow<'imm, ImmData, MutData>,
        RefMap:
            ReferenceMap<<ColWindow::Col as Keyable>::Key, (UnsafeIndex, Data<ImmData, MutData>)>,
    {
        colwindow: ColWindow,
        items: RefMap,
        phantom: PhantomData<&'imm (ImmData, MutData)>,
    }

    impl<'imm, ImmData, MutData, ColWindow, RefMap>
        CheckPrimary<'imm, ImmData, MutData, ColWindow, RefMap>
    where
        RefMap:
            ReferenceMap<<ColWindow::Col as Keyable>::Key, (UnsafeIndex, Data<ImmData, MutData>)>,
        ColWindow: PrimaryWindow<'imm, ImmData, MutData>,
        ImmData: Clone + Eq + std::fmt::Debug,
        MutData: Clone + Eq + std::fmt::Debug,
        <ColWindow::Col as Keyable>::Key: Eq + Hash,
    {
        fn new(size_hint: usize, colwindow: ColWindow) -> Self {
            Self {
                colwindow,
                items: RefMap::with_capacity(size_hint),
                phantom: PhantomData,
            }
        }

        fn check_get(&self, key: <ColWindow::Col as Keyable>::Key) {
            if let Some((unsafeindex, data)) = self.items.get(&key) {
                let entry = self
                    .colwindow
                    .get(key)
                    .expect("Key unexpectedly missing from column");
                let imm_data = ColWindow::conv_get(entry.data.imm_data);
                assert_eq!(imm_data, data.imm_data, "Incorrect immutable data");
                assert_eq!(entry.data.mut_data, data.mut_data, "Incorrect mutable data");
                assert_eq!(entry.index, *unsafeindex, "Incorrect index");
            } else {
                let entry = self.colwindow.get(key);
                assert!(entry.is_err(), "Key unexpectedly present in column");
            }
        }
    }

    impl<'imm, ImmData, MutData, ColWindow, RefMap>
        CheckPrimary<'imm, ImmData, MutData, ColWindow, RefMap>
    where
        RefMap:
            ReferenceMap<<ColWindow::Col as Keyable>::Key, (UnsafeIndex, Data<ImmData, MutData>)>,
        ColWindow: PrimaryWindowApp<'imm, ImmData, MutData>,
        ImmData: Clone + Eq + std::fmt::Debug,
        MutData: Clone + Eq + std::fmt::Debug,
        <ColWindow::Col as Keyable>::Key: Eq + Hash,
    {
        fn check_append(&mut self, data: Data<ImmData, MutData>) {
            let key = self.colwindow.append(data.clone());
            let unsafeindex = self.items.len();
            assert!(
                self.items
                    .insert(key, (unsafeindex, data.clone()))
                    .is_none(),
                "Key unexpectedly present in column"
            );
        }
    }

    impl<'imm, ImmData, MutData, ColWindow, RefMap>
        CheckPrimary<'imm, ImmData, MutData, ColWindow, RefMap>
    where
        RefMap:
            ReferenceMap<<ColWindow::Col as Keyable>::Key, (UnsafeIndex, Data<ImmData, MutData>)>,
        ColWindow: PrimaryWindowPull<'imm, ImmData, MutData>,
        ImmData: Clone + Eq + std::fmt::Debug,
        MutData: Clone + Eq + std::fmt::Debug,
        <ColWindow::Col as Keyable>::Key: Eq + Hash,
    {
        fn check_pull(&mut self, key: <ColWindow::Col as Keyable>::Key) {
            if let Some((unsafeindex, data)) = self.items.remove(&key) {
                let entry = self
                    .colwindow
                    .pull(key)
                    .expect("Key unexpectedly missing from column");
                let imm_data = ColWindow::conv_pull(entry.data.imm_data);
                assert_eq!(imm_data, data.imm_data, "Incorrect immutable data");
                assert_eq!(entry.data.mut_data, data.mut_data, "Incorrect mutable data");
                assert_eq!(entry.index, unsafeindex, "Incorrect index");
            } else {
                let entry = self.colwindow.pull(key);
                assert!(entry.is_err(), "Key unexpectedly present in column");
            }
        }

        fn check_insert(&mut self, data: Data<ImmData, MutData>) {
            let (key, action) = self.colwindow.insert(data.clone());
            match action {
                InsertAction::Place(unsafeindex) => {
                    assert!(
                        self.items
                            .insert(key, (unsafeindex, data.clone()))
                            .is_none(),
                        "Key unexpectedly present in column"
                    );
                }
                InsertAction::Append => {
                    let unsafeindex = self.items.len();
                    assert!(
                        self.items
                            .insert(key, (unsafeindex, data.clone()))
                            .is_none(),
                        "Key unexpectedly present in column"
                    );
                }
            }
        }
    }

    #[cfg(test)]
    mod test_verif {
        use super::*;

        fn check_primary_pull<Col>()
        where
            Col: Column,
            for<'a> Col::WindowKind<'a>: PrimaryWindowPull<'a, usize, usize>,
            for<'a> <<Col::WindowKind<'a> as PrimaryWindow<'a, usize, usize>>::Col as Keyable>::Key:
                Eq + Hash,
        {
            const ITERS: usize = 100000;
            let mut col = Col::new(ITERS);
            let mut check: CheckPrimary<_, _, _, HashMap<_, _>> =
                CheckPrimary::new(ITERS, col.window());

            for n in 0..1024 {
                check.check_insert(Data {
                    imm_data: n,
                    mut_data: n,
                });
                check.check_insert(Data {
                    imm_data: n,
                    mut_data: n,
                });
                if let Some(next_key) = check.items.get_next_key() {
                    check.check_get(next_key)
                }
                if let Some(next_key) = check.items.get_next_key() {
                    check.check_pull(next_key);
                    check.check_pull(next_key);
                }
            }
        }

        fn check_primary_app<Col>()
        where
            Col: Column,
            for<'a> Col::WindowKind<'a>: PrimaryWindowApp<'a, usize, usize>,
            for<'a> <<Col::WindowKind<'a> as PrimaryWindow<'a, usize, usize>>::Col as Keyable>::Key:
                Eq + Hash,
        {
            const ITERS: usize = 100000;
            let mut col = Col::new(ITERS);
            let mut check: CheckPrimary<_, _, _, HashMap<_, _>> =
                CheckPrimary::new(ITERS, col.window());

            for n in 0..1024 {
                check.check_append(Data {
                    imm_data: n,
                    mut_data: n,
                });
                check.check_append(Data {
                    imm_data: n,
                    mut_data: n,
                });
                if let Some(next_key) = check.items.get_next_key() {
                    check.check_get(next_key)
                }
            }
        }

        macro_rules! test_pull_impl {
            ($name:ident => $col:ty) => {
                #[test]
                fn $name() {
                    check_primary_pull::<$col>();
                }
            };
        }

        macro_rules! test_app_impl {
            ($name:ident => $col:ty) => {
                #[test]
                fn $name() {
                    check_primary_app::<$col>();
                }
            };
        }

        test_pull_impl!(primary_retain => PrimaryRetain<usize, usize, 16>);
        test_pull_impl!(gen_arena => PrimaryGenerationalArena<usize, usize>);
        test_pull_impl!(thunderdome => PrimaryThunderDome<usize, usize>);

        test_app_impl!(block => PrimaryAppend<AssocBlocks<usize, usize, 16>>);
    }

    #[cfg(kani)]
    mod kani_verif {
        use super::*;

        fn verif_pull<Col, const ITERS: usize>()
        where
            Col: Column,
            for<'a> Col::WindowKind<'a>: PrimaryWindowPull<'a, usize, usize>,
            for<'a> <Col::WindowKind<'a> as PrimaryWindow<'a, usize, usize>>::Key:
                kani::Arbitrary + Eq + Hash,
        {
            let mut col = Col::new(ITERS);
            let mut check: CheckPrimary<_, _, _, SimpleMap<_, _>> =
                CheckPrimary::new(ITERS, col.window());

            for n in 0..ITERS {
                check.check_insert(Data {
                    imm_data: n,
                    mut_data: n,
                });
                check.check_insert(Data {
                    imm_data: n,
                    mut_data: n,
                });
                check.check_pull(kani::any());
                check.check_get(kani::any());
            }
        }

        #[kani::proof]
        #[kani::unwind(6)]
        fn check_id_arena() {
            verif_pull::<PrimaryRetain<usize, usize, 16>, 5>();
        }
    }
}
