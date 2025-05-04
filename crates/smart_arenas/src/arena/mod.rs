use crate::key::{KeyTrait, WeakKey};

mod bind;
mod own;
mod share;

pub use bind::*;
pub use own::*;
pub use share::*;

/// An arena data structure, storing values associated with a container determined key.
/// - Capabilities of arenas are expressed through traits (for example [DeleteArena])
/// - All arenas use keys with the same [KeyTrait], separating the key type from the
///   arena allows for arenas containing complex self referential keys, or cyclic (between
///   arenas) keys.
pub trait Arena {
    type Key: KeyTrait;
    type Data;
    type Read<'a>
    where
        Self: 'a;

    fn new(preallocate_to: <Self::Key as KeyTrait>::Idx) -> Self;
    fn insert_return_reuse(&mut self, data: Self::Data) -> Option<(Self::Key, bool)>;
    fn insert(&mut self, data: Self::Data) -> Option<Self::Key> {
        self.insert_return_reuse(data).map(|(k, _)| k)
    }
    fn read(&self, key: &Self::Key) -> Self::Read<'_>;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (WeakKey<'a, Self::Key>, Self::Read<'a>)> + 'a;
}

/// An arena supporting deletion of keys.
pub trait DeleteArena: Arena {
    fn delete_return_dropped(&mut self, key: Self::Key) -> bool;
    fn delete(&mut self, key: Self::Key) {
        _ = self.delete_return_dropped(key);
    }
}

/// An arena supporting mutable values.
pub trait WriteArena: Arena {
    type Write<'a>
    where
        Self: 'a;
    fn write(&mut self, key: &Self::Key) -> Self::Write<'_>;
}

/// An arena that allows keys to be copied.
pub trait CopyKeyArena: Arena {
    fn copy_key(&mut self, key: &<Self as Arena>::Key) -> Option<<Self as Arena>::Key>;
}

/// Transform all members of an arena, consuming the arena, but leaving the keys as valid.
// TODO: Implement this trait, with in-place specialisation
pub trait TransformArena<InputData, OutputData>: Arena<Data = InputData> {
    fn transform(
        self,
        f: impl Fn(InputData) -> OutputData,
    ) -> impl Arena<Data = OutputData, Key = Self::Key>;
}

mod common {
    use crate::{
        alloc::AllocImpl,
        key::{IdxInt, KeyTrait, WeakKey},
    };
    use std::{cmp::Reverse, marker::PhantomData, mem::ManuallyDrop};

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

    pub struct Iter<
        'a,
        Key: KeyTrait,
        Data,
        Indexable: AllocImpl<Key::Idx, ValOrFree<Key::Idx, Data>>,
    > {
        alloc: &'a Indexable,
        sorted_desc_free: Vec<Key::Idx>,
        maybe_free_current: Key::Idx,
        _phantom: PhantomData<Data>,
    }

    impl<'a, Key: KeyTrait, Data, Indexable: AllocImpl<Key::Idx, ValOrFree<Key::Idx, Data>>>
        Iter<'a, Key, Data, Indexable>
    {
        pub fn new(alloc: &'a Indexable, next_free: Option<Key::Idx>, used_len: usize) -> Self {
            Self {
                alloc,
                sorted_desc_free: unsafe { collect_sorted_frees(next_free, used_len, alloc) },
                maybe_free_current: Key::Idx::ZERO,
                _phantom: PhantomData,
            }
        }
    }

    impl<
        'a,
        Key: KeyTrait + 'a,
        Data: 'a,
        Indexable: AllocImpl<Key::Idx, ValOrFree<Key::Idx, Data>>,
    > Iterator for Iter<'a, Key, Data, Indexable>
    {
        type Item = (WeakKey<'a, Key>, &'a Data);

        fn next(&mut self) -> Option<Self::Item> {
            while Some(&self.maybe_free_current) == self.sorted_desc_free.last() {
                self.sorted_desc_free.pop();
                self.maybe_free_current.inc();
            }

            if self.maybe_free_current.offset() < self.alloc.len() {
                Some(unsafe {
                    (
                        WeakKey::new_from_idx(self.maybe_free_current),
                        &self.alloc.read(self.maybe_free_current).data,
                    )
                })
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        alloc,
        arena::{self, Arena, DeleteArena},
        key,
    };
    #[test]
    fn foo() {
        struct Tag;
        type Key = key::Key<Tag, u16>;
        {
            let mut arena: arena::Own<Key, alloc::Contig, usize> = arena::Own::new(0);
            let out = arena.insert(12).unwrap();
            assert_eq!(arena.read(&out), &12);
            arena.delete(out);
        }

        {
            let mut arena2: arena::Own<Key, alloc::Contig, usize> = arena::Own::new(0);
            let out = arena2.insert(15).unwrap();
            assert_eq!(arena2.read(&out), &12);
            arena2.delete(out);
        }
    }
}
