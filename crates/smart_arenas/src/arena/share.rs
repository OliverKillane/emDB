use super::{Arena, CopyKeyArena, DeleteArena, WriteArena, common};
use crate::{
    alloc::{AllocImpl, AllocSelect},
    id::{
        index::{Index, WidestIndex},
        key::{KeyTrait, WeakKey},
        token::Token,
    },
};
use std::mem::ManuallyDrop;

/// ## Shared Ownership Arena
/// The equivalent of [std::rc::Rc]/[std::sync::Arc] for arenas.
pub struct Share<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, RefCount: Index, Data> {
    data_slots: Alloc::Impl<Key::Idx, common::ValOrFree<Key::Idx, Data>>,
    refcount_slots: Alloc::Impl<Key::Idx, RefCount>,
    next_free: Option<Key::Idx>,
    len: usize,
    _token: Token<'id>,
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, RefCount: Index, Data>
    Share<'id, Key, Alloc, RefCount, Data>
{
    fn slots_exclusive_upper_bound(&self) -> WidestIndex {
        debug_assert!(
            self.data_slots.exclusive_index_upper_bound()
                == self.refcount_slots.exclusive_index_upper_bound()
        );
        self.data_slots.exclusive_index_upper_bound()
    }
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, RefCount: Index, Data> Arena<'id>
    for Share<'id, Key, Alloc, RefCount, Data>
{
    type Key = Key;
    type Data = Data;
    type Read<'a>
        = &'a Data
    where
        Self: 'a;

    fn new(preallocate_to: <Self::Key as KeyTrait<'id>>::Idx, token: Token<'id>) -> Self {
        Self {
            data_slots: Alloc::Impl::new(preallocate_to),
            refcount_slots: Alloc::Impl::new(preallocate_to),
            next_free: None,
            len: 0,
            _token: token,
        }
    }

    fn insert_return_reuse(&mut self, data: Data) -> Option<(Self::Key, bool)> {
        if let Some(idx) = self.next_free {
            unsafe {
                let data_slot = self.data_slots.write(idx);
                let refcount_slot = self.refcount_slots.write(idx);
                self.next_free = *data_slot.next_free;
                ManuallyDrop::drop(&mut data_slot.next_free);
                data_slot.data = ManuallyDrop::new(data);
                *refcount_slot = RefCount::ZERO.inc();
            }
            Some((idx, true))
        } else if let Some(idx) = self.data_slots.append(common::ValOrFree {
            data: ManuallyDrop::new(data),
        }) {
            self.refcount_slots.append(RefCount::ZERO.inc());
            Some((idx, false))
        } else {
            None
        }
        .map(|(idx, reused)| {
            self.len += 1;
            unsafe { (Key::to_key(idx), reused) }
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
        unsafe { &self.data_slots.read(key.to_idx()).data }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn iter_with_weak_key<'a>(
        &'a self,
    ) -> impl Iterator<Item = (WeakKey<'id, 'a, Self::Key>, Self::Read<'a>)> + 'a {
        ShareIter {
            arena: self,
            current: Key::Idx::ZERO,
        }
    }

    fn insert(&mut self, data: Self::Data) -> Option<Self::Key> {
        self.insert_return_reuse(data).map(|(k, _)| k)
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, RefCount: Index, Data> DeleteArena<'id>
    for Share<'id, Key, Alloc, RefCount, Data>
{
    fn delete_return_dropped(&mut self, key: Self::Key) -> bool {
        let dropped = unsafe {
            let data_slot = self.data_slots.write(key.to_idx());
            let refcount_slot = self.refcount_slots.write(key.to_idx());

            *refcount_slot = refcount_slot.dec();
            if *refcount_slot == RefCount::ZERO {
                ManuallyDrop::drop(&mut data_slot.data);
                data_slot.next_free = ManuallyDrop::new(self.next_free);
                self.next_free = Some(key.to_idx());
                self.len -= 1;
                true
            } else {
                false
            }
        };
        dropped
    }
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, RefCount: Index, Data> WriteArena<'id>
    for Share<'id, Key, Alloc, RefCount, Data>
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
        unsafe { &mut self.data_slots.write(key.to_idx()).data }
    }
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, RefCount: Index, Data> CopyKeyArena<'id>
    for Share<'id, Key, Alloc, RefCount, Data>
{
    fn copy_key(&mut self, key: &Self::Key) -> Option<Self::Key> {
        unsafe {
            let entry = self.refcount_slots.write(key.to_idx());
            if *entry != RefCount::MAX {
                *entry = entry.inc();
                Some(Key::to_key(key.to_idx()))
            } else {
                None
            }
        }
    }
}
impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, RefCount: Index, Data> Drop
    for Share<'id, Key, Alloc, RefCount, Data>
{
    fn drop(&mut self) {
        for idx in 0..self.slots_exclusive_upper_bound() {
            let idx = Key::Idx::from_offset(idx).unwrap();
            if unsafe { self.refcount_slots.read(idx) } != &RefCount::ZERO {
                unsafe {
                    ManuallyDrop::drop(&mut self.data_slots.write(idx).data);
                }
            }
        }
    }
}

struct ShareIter<'id, 'brw, Key: KeyTrait<'id>, Alloc: AllocSelect, RefCount: Index, Data> {
    arena: &'brw Share<'id, Key, Alloc, RefCount, Data>,
    current: Key::Idx,
}

impl<'id, 'brw, Key: KeyTrait<'id>, Alloc: AllocSelect, RefCount: Index, Data> Iterator
    for ShareIter<'id, 'brw, Key, Alloc, RefCount, Data>
{
    type Item = (WeakKey<'id, 'brw, Key>, &'brw Data);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current.offset() >= self.arena.slots_exclusive_upper_bound() {
                break None;
            } else if unsafe { self.arena.refcount_slots.read(self.current) } != &RefCount::ZERO {
                unsafe {
                    let data = self.arena.read(&Key::to_key(self.current));
                    let weak_key = WeakKey::to_key(self.current);
                    self.current = self.current.inc();
                    break Some((weak_key, data));
                }
            } else {
                self.current = self.current.inc();
            }
        }
    }
}
