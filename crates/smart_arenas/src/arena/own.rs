use super::{Arena, DeleteArena, WriteArena, common};
use crate::{
    alloc::{AllocImpl, AllocSelect},
    key::{KeyTrait, WeakKey},
};
use std::mem::ManuallyDrop;

pub struct Own<Key: KeyTrait, Alloc: AllocSelect, Data> {
    slots: Alloc::Impl<Key::Idx, common::ValOrFree<Key::Idx, Data>>,
    next_free: Option<Key::Idx>,
    len: usize,
}

impl<Key: KeyTrait, Alloc: AllocSelect, Data> Drop for Own<Key, Alloc, Data> {
    fn drop(&mut self) {
        if !self.is_empty() {
            panic!("Some values are still in the arena, meaning a leak has occured")
        }
        unsafe {
            Key::relinquish();
        }
    }
}

impl<Key: KeyTrait, Alloc: AllocSelect, Data> Arena for Own<Key, Alloc, Data> {
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
                slot.data = ManuallyDrop::new(data);
            }
            self.len += 1;
            Some((idx, true))
        } else if let Some(idx) = self.slots.append(common::ValOrFree {
            data: ManuallyDrop::new(data),
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
        unsafe { &self.slots.read(key.to_idx()).data }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (WeakKey<'a, Self::Key>, Self::Read<'a>)> {
        common::Iter::new(&self.slots, self.next_free, self.len())
    }
}

impl<Key: KeyTrait, Alloc: AllocSelect, Data> DeleteArena for Own<Key, Alloc, Data> {
    fn delete_return_dropped(&mut self, key: Self::Key) -> bool {
        unsafe {
            let value = self.slots.write(key.to_idx());
            ManuallyDrop::drop(&mut value.data);
            value.next_free = ManuallyDrop::new(self.next_free);
        }
        self.next_free = Some(key.to_idx());
        key.dispose();
        self.len -= 1;
        true
    }
}

impl<Key: KeyTrait, Alloc: AllocSelect, Data> WriteArena for Own<Key, Alloc, Data> {
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
