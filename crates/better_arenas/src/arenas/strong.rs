use super::{
    common::{Cfg, Key, ValOrFree}, Arena, CopyKeyArena, WriteArena
};
use crate::{
    alloc::{AllocImpl, AllocSelect},
    utils::{idx::IdxInt, unique::UniqueToken},
};
use std::{marker::PhantomData, mem::ManuallyDrop};

pub struct Strong<Idx: IdxInt, Data, Alloc: AllocSelect, Id: UniqueToken, RefCount: IdxInt> {
    slots: Alloc::Impl<Idx, ValOrFree<Idx, (Data, RefCount)>>,
    next_free: Option<Idx>,
    _phantom: PhantomData<Id>,
}

impl<Idx: IdxInt, Data, Alloc: AllocSelect, Id: UniqueToken, RefCount: IdxInt> Arena<Data>
    for Strong<Idx, Data, Alloc, Id, RefCount>
{
    type Cfg = Cfg<
        <Alloc::Impl<Idx, ValOrFree<Idx, (Data, RefCount)>> as AllocImpl<
            Idx,
            ValOrFree<Idx, (Data, RefCount)>,
        >>::Cfg,
        Id,
    >;
    type Key = Key<Idx, Id>;

    fn new(Self::Cfg { unique: _, alloc }: Self::Cfg) -> Self {
        Self {
            slots: Alloc::Impl::new(alloc),
            next_free: None,
            _phantom: PhantomData,
        }
    }

    fn insert(&mut self, data: Data) -> Option<Self::Key> {
        if let Some(idx) = self.next_free {
            unsafe {
                let slot = self.slots.write(idx);
                self.next_free = *slot.next_free;
                ManuallyDrop::drop(&mut slot.next_free);
                slot.data = ManuallyDrop::new((data, RefCount::ZERO));
            }
            Some(Key::new(idx))
        } else {
            self.slots
                .insert(ValOrFree {
                    data: ManuallyDrop::new((data, RefCount::ZERO)),
                })
                .map(Key::new)
        }
    }

    fn read(&self, key: &Self::Key) -> &Data {
        // JUSTIFY: No bounds check on lookup of the key.
        //           - Keys can only be created from this module
        //           - Keys cannot be copied
        //           - Keys include a unique type marker, checked at construction.
        //          Hence it is only possible use a key, if it has been provided by this specific
        //          instance.
        // JUSTIFY: No check on union.
        //           - Keys cannot be copied, and deletion takes ownership of a key
        //          Hence this key must have been from an insert, and cannot have been deleted.
        unsafe { &self.slots.read(key.0.idx).data.0 }
    }

    fn delete(&mut self, key: Self::Key) {
        unsafe {
            let entry = self.slots.write(key.0.idx);
            entry.data.1 = entry.data.1.dec();
            if entry.data.1 == RefCount::ZERO {
                ManuallyDrop::drop(&mut entry.data);
                entry.next_free = ManuallyDrop::new(self.next_free);
                self.next_free = Some(key.0.idx);
            }
        }
    }
}

impl<Idx: IdxInt, Data, Alloc: AllocSelect, Id: UniqueToken, RefCount: IdxInt> WriteArena<Data>
    for Strong<Idx, Data, Alloc, Id, RefCount>
{
    fn write(&mut self, key: &Self::Key) -> &mut Data {
        // JUSTIFY: No bounds check on lookup of the key.
        //           - Keys can only be created from this module
        //           - Keys cannot be copied
        //           - Keys include a unique type marker, checked at construction.
        //          Hence it is only possible use a key, if it has been provided by this specific
        //          instance.
        // JUSTIFY: No check on union.
        //           - Keys cannot be copied, and deletion takes ownership of a key
        //          Hence this key must have been from an insert, and cannot have been deleted.
        unsafe { &mut self.slots.write(key.0.idx).data.0 }
    }
}

impl<Idx: IdxInt, Data, Alloc: AllocSelect, Id: UniqueToken, RefCount: IdxInt> CopyKeyArena<Data>
    for Strong<Idx, Data, Alloc, Id, RefCount>
{
    fn copy_key(&mut self, key: &<Self as Arena<Data>>::Key) -> Option<<Self as Arena<Data>>::Key> {
        unsafe {
            let entry = &mut self.slots.write(key.0.idx).data;
            if entry.1 != RefCount::MAX {
                entry.1 = entry.1.inc();
                Some(Key::new(key.0.idx))
            } else {
                None
            }
        }
    }
}
