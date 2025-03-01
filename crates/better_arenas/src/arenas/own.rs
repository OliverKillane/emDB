use super::{
    common::{Cfg, Key, ValOrFree}, Arena, WriteArena
};
use crate::{
    alloc::{AllocImpl, AllocSelect},
    utils::{idx::IdxInt, unique::UniqueToken},
};
use std::{marker::PhantomData, mem::ManuallyDrop};

struct Own<Idx: IdxInt, Data, Alloc: AllocSelect, Id: UniqueToken> {
    slots: Alloc::Impl<Idx, ValOrFree<Idx, Data>>,
    next_free: Option<Idx>,
    _phantom: PhantomData<Id>,
}

impl<Idx: IdxInt, Data, Alloc: AllocSelect, Id: UniqueToken> Arena<Data>
    for Own<Idx, Data, Alloc, Id>
{
    type Cfg = Cfg<
        <Alloc::Impl<Idx, ValOrFree<Idx, Data>> as AllocImpl<Idx, ValOrFree<Idx, Data>>>::Cfg,
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
                slot.data = ManuallyDrop::new(data);
            }
            Some(Key::new(idx))
        } else {
            self.slots
                .insert(ValOrFree {
                    data: ManuallyDrop::new(data),
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
        unsafe { &self.slots.read(key.0.idx).data }
    }

    fn delete(&mut self, key: Self::Key) {
        unsafe {
            let value = self.slots.write(key.0.idx);
            ManuallyDrop::drop(&mut value.data);
            value.next_free = ManuallyDrop::new(self.next_free);
        }
        self.next_free = Some(key.0.idx);
        key.0.drop(());
    }
}

impl<Idx: IdxInt, Data, Alloc: AllocSelect, Id: UniqueToken> WriteArena<Data>
    for Own<Idx, Data, Alloc, Id>
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
        unsafe { &mut self.slots.write(key.0.idx).data }
    }
}
