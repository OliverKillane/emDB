use super::{Arena, CopyKeyArena, DeleteArena, WriteArena, common};
use crate::{
    alloc::{AllocImpl, AllocSelect},
    key::{IdxInt, KeyTrait, WeakKey},
};
use std::mem::ManuallyDrop;

pub struct Share<Key: KeyTrait, Alloc: AllocSelect, RefCount: IdxInt, Data> {
    slots: Alloc::Impl<Key::Idx, common::ValOrFree<Key::Idx, (Data, RefCount)>>,
    next_free: Option<Key::Idx>,
    len: usize,
}

impl<Key: KeyTrait, Alloc: AllocSelect, RefCount: IdxInt, Data> Arena
    for Share<Key, Alloc, RefCount, Data>
{
    type Key = Key;
    type Data = Data;
    type Read<'a>
        = &'a Data
    where
        Self: 'a;

    fn new(preallocate_to: <Self::Key as KeyTrait>::Idx) -> Self {
        Self::Key::guard();
        Self {
            slots: Alloc::Impl::new(preallocate_to),
            next_free: None,
            len: 0,
        }
    }

    fn insert_return_reuse(&mut self, data: Data) -> Option<(Self::Key, bool)> {
        if let Some(idx) = self.next_free {
            unsafe {
                let slot = self.slots.write(idx);
                self.next_free = *slot.next_free;
                ManuallyDrop::drop(&mut slot.next_free);
                slot.data = ManuallyDrop::new((data, RefCount::ZERO));
            }
            self.len += 1;
            Some((idx, true))
        } else if let Some(idx) = self.slots.append(common::ValOrFree {
            data: ManuallyDrop::new((data, RefCount::ZERO)),
        }) {
            self.len += 1;
            Some((idx, false))
        } else {
            None
        }
        .map(|(idx, reused)| unsafe { (Key::to_key(idx), reused) })
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
        unsafe { &self.slots.read(key.to_idx()).data.0 }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (WeakKey<'a, Self::Key>, Self::Read<'a>)> {
        common::Iter::new(&self.slots, self.next_free, self.len()).map(|(wk, (data, _))| (wk, data))
    }
}

impl<Key: KeyTrait, Alloc: AllocSelect, RefCount: IdxInt, Data> DeleteArena
    for Share<Key, Alloc, RefCount, Data>
{
    fn delete_return_dropped(&mut self, key: Self::Key) -> bool {
        let dropped = unsafe {
            let entry = self.slots.write(key.to_idx());
            entry.data.1 = entry.data.1.dec();
            if entry.data.1 == RefCount::ZERO {
                ManuallyDrop::drop(&mut entry.data);
                entry.next_free = ManuallyDrop::new(self.next_free);
                self.next_free = Some(key.to_idx());
                self.len -= 1;
                true
            } else {
                false
            }
        };
        key.dispose();
        dropped
    }
}

impl<Key: KeyTrait, Alloc: AllocSelect, RefCount: IdxInt, Data> WriteArena
    for Share<Key, Alloc, RefCount, Data>
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
        unsafe { &mut self.slots.write(key.to_idx()).data.0 }
    }
}

impl<Key: KeyTrait, Alloc: AllocSelect, RefCount: IdxInt, Data> CopyKeyArena
    for Share<Key, Alloc, RefCount, Data>
{
    fn copy_key(&mut self, key: &Self::Key) -> Option<Self::Key> {
        unsafe {
            let entry = &mut self.slots.write(key.to_idx()).data;
            if entry.1 != RefCount::MAX {
                entry.1 = entry.1.inc();
                Some(Key::to_key(key.to_idx()))
            } else {
                None
            }
        }
    }
}
