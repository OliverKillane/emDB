use crate::{
    alloc::{AllocImpl, AllocSelect},
    arena::Arena,
    id::{
        index::{Index, WidestIndex},
        key::{KeyTrait, WeakKey},
        token::Token,
    },
};

use super::{CopyKeyArena, IterKeyArena, WriteArena};

/// ## An Append only arena
///  - No support for deletions, all values remain in the map until the arena is deallocated
pub struct App<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> {
    slots: Alloc::Impl<Key::Idx, Data>,
    _token: Token<'id>,
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> Arena<'id> for App<'id, Key, Alloc, Data> {
    type Key = Key;
    type Data = Data;
    type Read<'a>
        = &'a Data
    where
        Self: 'a;

    fn new(preallocate_to: <Self::Key as KeyTrait<'id>>::Idx, token: Token<'id>) -> Self {
        Self {
            slots: Alloc::Impl::new(preallocate_to),
            _token: token,
        }
    }

    fn insert_return_reuse(&mut self, data: Self::Data) -> Option<(Self::Key, bool)> {
        self.slots
            .append(data)
            .map(|idx| (unsafe { Self::Key::to_key(idx) }, false))
    }

    fn read(&self, key: &Self::Key) -> Self::Read<'_> {
        unsafe { &self.slots.read(key.to_idx()) }
    }

    fn len(&self) -> usize {
        self.slots.exclusive_index_upper_bound() as usize
    }

    fn iter_with_weak_key<'a>(
        &'a self,
    ) -> impl Iterator<Item = (crate::prelude::WeakKey<'id, 'a, Self::Key>, Self::Read<'a>)> + 'a
    {
        (0..self.len()).map(|offset| {
            let idx = Key::Idx::from_offset(offset as WidestIndex).unwrap();
            unsafe {
                let weak_key = WeakKey::to_key(idx);
                let data = self.slots.read(idx);
                (weak_key, data)
            }
        })
    }
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> WriteArena<'id>
    for App<'id, Key, Alloc, Data>
{
    type Write<'a>
        = &'a mut Data
    where
        Self: 'a;

    fn write(&mut self, key: &Self::Key) -> Self::Write<'_> {
        unsafe { self.slots.write(key.to_idx()) }
    }
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> CopyKeyArena<'id>
    for App<'id, Key, Alloc, Data>
{
    fn copy_key(&mut self, key: &<Self as Arena<'id>>::Key) -> Option<<Self as Arena<'id>>::Key> {
        Some(unsafe { Self::Key::to_key(key.to_idx()) })
    }
}

impl<'id, Key: KeyTrait<'id>, Alloc: AllocSelect, Data> IterKeyArena<'id>
    for App<'id, Key, Alloc, Data>
{
    fn iter_with_key<'a>(&'a self) -> impl Iterator<Item = (Self::Key, Self::Read<'a>)> + 'a {
        self.iter_with_weak_key()
            .map(|(weak_key, data)| (unsafe { Self::Key::to_key(weak_key.to_idx()) }, data))
    }
}
