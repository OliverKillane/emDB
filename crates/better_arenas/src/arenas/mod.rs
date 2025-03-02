//! ## Arena Data Structures
//!
//! ## TODO:
//! ### Interned Arena
//! An interned arena type (need to implement a special hashmap allowing equality
//! with context - so we can just store small indices in the map, have equality
//! lookup the rest of the arena).
//!
//! ### Weak Arena
//! Allow a second key type that is an observer for a [strong::Strong] arena.

pub mod own;
pub mod strong;

/// A type-level function, that allows types stored inside an [Arena] to use
/// the key type of the arena
///  - Without this we get a cycle in the bounds of an arena type when it's own
///    key is used in it's data (e.g. some `X: Arena<SomeOtherData<X::Key>>` bound)
///
/// Key types need to be accessed from the arena type, as they often contain
/// anonymous types, so there is no syntax to refer to them directly.
pub trait Store {
    type Data<Key>;
}

pub struct Plain<Data>(std::marker::PhantomData<Data>);

impl<Data> Store for Plain<Data> {
    type Data<Key> = Data;
}

/// An arena data structure, storing values associated with a container determined key.
///
/// ## Traits
/// Capabilities of arenas are expressed through traits.
///  - See additionally [WriteArena], [DeleteArena], [CopyKeyArena]
///
/// ## [Store]
/// The data type stored in the arena is determined by the [Store] trait.
///
/// We can use [Plain] to just store a data type.
/// ```
/// # use better_arenas::prelude::*;
/// struct ExampleData {
///     foo: i32,
///     bar: [u8; 4]
/// }
///
/// fn read_insert<A: Arena<Plain<ExampleData>>>(a: &mut A, key: &A::Key) -> Option<A::Key> {
///     let data: &ExampleData = a.read(key);
///     a.insert(ExampleData { foo: 3, bar: [1, 2, 3, 4] })
/// }
/// ```
///
/// However if the data type needs to contain the key type of the arena it is
/// to be stored in, we need to implement our own store.
/// ```
/// # use better_arenas::prelude::*;
/// struct ExampleData<OtherKey> {
///     foo: i32,
///     bar: [u8; 4],
///     other_key: Option<OtherKey>
/// }
///
/// struct ExampleStore;
///
/// impl Store for ExampleStore {
///     type Data<Key> = ExampleData<Key>;
/// }
///
/// fn update_key<A: Arena<ExampleStore>>(a: &mut A, key: &A::Key) {
///     let data: &ExampleData<A::Key> = a.read(key);
///     // ... do something with the data
/// }
/// ```
///
/// ## Arena Behaviour
/// Arenas can choose different behaviour with regards to keys (droppability,
/// equality semantics).
///  - [own::Own] - Keys are not droppable, and are not copyable.
///  - [strong::Strong] - Keys not droppable, and are copyable (see [CopyKeyArena]).
pub trait Arena<S: Store> {
    type Cfg;
    type Key: std::hash::Hash + Eq;

    fn new(cfg: Self::Cfg) -> Self;
    fn insert(&mut self, data: S::Data<Self::Key>) -> Option<Self::Key>;
    fn read(&self, key: &Self::Key) -> &S::Data<Self::Key>;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// An arena supporting deletion of keys.
pub trait DeleteArena<S: Store>: Arena<S> {
    fn delete(&mut self, key: Self::Key);
}

/// An arena supporting mutable values.
pub trait WriteArena<S: Store>: Arena<S> {
    fn write(&mut self, key: &Self::Key) -> &mut S::Data<Self::Key>;
}

/// An arena that allows keys to be copied.
pub trait CopyKeyArena<S: Store>: Arena<S> {
    fn copy_key(&mut self, key: &<Self as Arena<S>>::Key) -> Option<<Self as Arena<S>>::Key>;
}

// JUSTIFY: Not owning, or mutable iteration
//           - Arenas can define their own semantics for keys, hence an owning
//             `into_iter` would require users to [std::mem::forget] or never
//             drop their keys.
//           - Having a guarentee that owning a key for [own::Own] is exclusive
//             ownership (no mutation) is useful.

/// An arena allowing iteration over references to all values.
///
/// ```
/// use better_arenas::prelude::*;
/// struct ExampleData(i32);
///
/// fn iter_over_arena(a: &impl IterArena<Plain<ExampleData>>) -> impl Iterator<Item = &i32> {
///     a.iter().map(|ExampleData(x)| x)
/// }
/// ```
pub trait IterArena<S: Store>: Arena<S> {
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a S::Data<Self::Key>> + 'a
    where
        <S as Store>::Data<<Self as Arena<S>>::Key>: 'a;
}

mod common {
    use derive_where::derive_where;

    use crate::{
        alloc::AllocImpl,
        utils::{
            drop::{CanDropWith, DropWith},
            idx::IdxInt,
            unique::UniqueToken,
        },
    };
    use std::{cmp::Reverse, marker::PhantomData, mem::ManuallyDrop};

    #[derive_where(Hash, PartialEq, Eq)]
    pub struct KeyInner<Idx: IdxInt, Id: UniqueToken> {
        pub idx: Idx,
        _phantom: PhantomData<Id>,
    }

    impl<Idx: IdxInt, Id: UniqueToken> CanDropWith<()> for KeyInner<Idx, Id> {
        fn drop(self, _: ()) {}
    }

    #[derive_where(Hash, PartialEq, Eq)]
    pub struct Key<Idx: IdxInt, Id: UniqueToken>(pub DropWith<KeyInner<Idx, Id>>);

    impl<Idx: IdxInt, Id: UniqueToken> Key<Idx, Id> {
        pub fn new(idx: Idx) -> Self {
            Self(DropWith::new(KeyInner {
                idx,
                _phantom: PhantomData,
            }))
        }
    }

    pub union ValOrFree<Idx: IdxInt, Data> {
        pub data: ManuallyDrop<Data>,
        pub next_free: ManuallyDrop<Option<Idx>>,
    }

    pub unsafe fn collect_sorted_frees<
        Idx: IdxInt,
        Data,
        Indexable: super::super::alloc::AllocImpl<Idx, ValOrFree<Idx, Data>>,
    >(
        start: Option<Idx>,
        expected_len: usize,
        alloc: &Indexable,
    ) -> Vec<Idx> {
        if let Some(mut current) = start {
            let mut free_unsorted = Vec::with_capacity(expected_len);
            free_unsorted.push(current);

            while let Some(next_free) = { unsafe { alloc.read(current).next_free.as_ref() } } {
                current = *next_free;
                free_unsorted.push(*next_free);
            }

            free_unsorted.sort_unstable_by_key(|&k| Reverse(k));
            free_unsorted
        } else {
            Vec::new()
        }
    }

    pub struct Iter<'a, Idx: IdxInt, Data, Indexable: AllocImpl<Idx, ValOrFree<Idx, Data>>> {
        alloc: &'a Indexable,
        sorted_desc_free: Vec<Idx>,
        maybe_free_current: Idx,
        _phantom: PhantomData<Data>,
    }

    impl<'a, Idx: IdxInt, Data, Indexable: AllocImpl<Idx, ValOrFree<Idx, Data>>>
        Iter<'a, Idx, Data, Indexable>
    {
        pub fn new(alloc: &'a Indexable, next_free: Option<Idx>, used_len: usize) -> Self {
            Self {
                alloc,
                sorted_desc_free: unsafe { collect_sorted_frees(next_free, used_len, alloc) },
                maybe_free_current: Idx::ZERO,
                _phantom: PhantomData,
            }
        }
    }

    impl<'a, Idx: IdxInt + 'a, Data: 'a, Indexable: AllocImpl<Idx, ValOrFree<Idx, Data>>> Iterator
        for Iter<'a, Idx, Data, Indexable>
    {
        type Item = &'a Data;

        fn next(&mut self) -> Option<Self::Item> {
            while Some(&self.maybe_free_current) == self.sorted_desc_free.last() {
                self.sorted_desc_free.pop();
                self.maybe_free_current.inc();
            }

            if self.maybe_free_current.offset() < self.alloc.len() {
                Some(unsafe { &self.alloc.read(self.maybe_free_current).data })
            } else {
                None
            }
        }
    }
}
