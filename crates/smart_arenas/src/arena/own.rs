use super::{Arena, DeleteArena, WriteArena, common};
use crate::{
    alloc::{AllocImpl, AllocSelect},
    id::{
        index::Index,
        key::{KeyTrait, WeakKey},
        token::Token,
    },
};
use roaring::RoaringBitmap;
use std::mem::ManuallyDrop;

/// ## Unique Ownership Arena
/// The equivalent of [Box] for arenas.
///  - Only one key per value, keys cannot be copied.
pub struct Own<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> {
    slots: Alloc::Impl<Key::Idx, common::ValOrFree<Key::Idx, Data>>,
    next_free: Option<Key::Idx>,
    deleted_slots: RoaringBitmap,
    len: usize,
    _token: Token<'id>,
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> Arena<'id> for Own<'id, Key, Alloc, Data> {
    type Key = Key;
    type Data = Data;
    type Read<'a>
        = &'a Data
    where
        Self: 'a;

    fn new(preallocate_to: <Self::Key as KeyTrait<'id>>::Idx, token: Token<'id>) -> Self {
        Self {
            slots: Alloc::Impl::new(preallocate_to),
            next_free: None,
            len: 0,
            deleted_slots: RoaringBitmap::new(),
            _token: token,
        }
    }

    fn insert_return_reuse(&mut self, data: Data) -> Option<(Self::Key, bool)> {
        if let Some(idx) = self.next_free {
            unsafe {
                let slot = self.slots.write(idx);
                self.next_free = *slot.next_free;
                ManuallyDrop::drop(&mut slot.next_free);
                slot.data = ManuallyDrop::new(data);
            }
            Some((idx, true))
        } else {
            self.slots
                .append(common::ValOrFree {
                    data: ManuallyDrop::new(data),
                })
                .map(|idx| (idx, false))
        }
        .map(|(idx, reused)| {
            self.len += 1;
            let _ = self.deleted_slots.remove(idx.offset());
            unsafe { (Key::from_idx(idx), reused) }
        })
    }

    fn read(&self, key: &Self::Key) -> Self::Read<'_> {
        // JUSTIFY: No bounds check on lookup of the key.
        //           - Keys can only be created from this module
        //           - Keys cannot be copied
        //           - Keys include a unique type marker, checked at construction.
        //          Hence it is only possible use a key, if it has been provided by this specific
        //          instance.
        // JUSTIFY: No check on union.
        //           - Keys cannot be copied, and deletion takes ownership of a key
        //          Hence this key must have been from an insert, and cannot have been deleted.
        debug_assert!(!self.deleted_slots.contains(key.to_idx().offset()));
        unsafe { &self.slots.read(key.to_idx()).data }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn iter_with_weak_key<'a>(
        &'a self,
    ) -> impl Iterator<Item = (WeakKey<'id, 'a, Self::Key>, Self::Read<'a>)> + 'a {
        OwnIter {
            arena: self,
            current: Key::Idx::ZERO,
        }
    }
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> DeleteArena<'id>
    for Own<'id, Key, Alloc, Data>
{
    fn delete_return_dropped(&mut self, key: Self::Key) -> bool {
        unsafe {
            let value = self.slots.write(key.to_idx());
            ManuallyDrop::drop(&mut value.data);
            value.next_free = ManuallyDrop::new(self.next_free);
        }
        self.next_free = Some(key.to_idx());
        self.deleted_slots.insert(key.to_idx().offset());
        self.len -= 1;
        true
    }
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> WriteArena<'id>
    for Own<'id, Key, Alloc, Data>
{
    type Write<'a>
        = &'a mut Data
    where
        Self: 'a;
    fn write(&mut self, key: &Self::Key) -> Self::Write<'_> {
        // JUSTIFY: No bounds check on lookup of the key.
        //           - Keys can only be created from this module
        //           - Keys cannot be copied
        //           - Keys include a unique type marker, checked at construction.
        //          Hence it is only possible use a key, if it has been provided by this specific
        //          instance.
        // JUSTIFY: No check on union.
        //           - Keys cannot be copied, and deletion takes ownership of a key
        //          Hence this key must have been from an insert, and cannot have been deleted.
        unsafe { &mut self.slots.write(key.to_idx()).data }
    }
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> Drop for Own<'id, Key, Alloc, Data> {
    fn drop(&mut self) {
        for idx in 0..self.slots.exclusive_index_upper_bound() {
            if !self.deleted_slots.contains(idx) {
                unsafe {
                    ManuallyDrop::drop(
                        &mut self.slots.write(Key::Idx::from_offset(idx).unwrap()).data,
                    )
                }
            }
        }
    }
}

struct OwnIter<'id, 'brw, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> {
    arena: &'brw Own<'id, Key, Alloc, Data>,
    current: Key::Idx,
}

impl<'id, 'brw, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> Iterator
    for OwnIter<'id, 'brw, Key, Alloc, Data>
{
    type Item = (WeakKey<'id, 'brw, Key>, &'brw Data);

    fn next(&mut self) -> Option<Self::Item> {
        // JUSTIFY: Checking deleted slots first
        //           - if deleted, then the slot was allocated, so was present
        //           - no access before the exclusive upper bound check
        while self.arena.deleted_slots.contains(self.current.offset()) {
            self.current = self.current.inc();
        }
        if self.current.offset() >= self.arena.slots.exclusive_index_upper_bound() {
            None
        } else {
            unsafe {
                let data = self.arena.read(&Key::from_idx(self.current));
                let weak_key = WeakKey::from_idx(self.current);
                self.current = self.current.inc();
                Some((weak_key, data))
            }
        }
    }
}
