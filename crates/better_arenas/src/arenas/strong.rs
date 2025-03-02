use super::{
    Arena, CopyKeyArena, DeleteArena, Store, WriteArena,
    common::{self, Key, ValOrFree},
};
use crate::{
    alloc::{AllocImpl, AllocSelect},
    utils::{idx::IdxInt, unique::UniqueToken},
};
use std::{marker::PhantomData, mem::ManuallyDrop};

pub struct Strong<Idx: IdxInt, S: Store, Alloc: AllocSelect, Id: UniqueToken, RefCount: IdxInt> {
    // JUSTIFY: Complex type
    //          Splitting this into a type alias would not make it simpler.
    #[allow(clippy::type_complexity)]
    slots: Alloc::Impl<
        Idx,
        ValOrFree<
            Idx,
            (
                S::Data<<Strong<Idx, S, Alloc, Id, RefCount> as Arena<S>>::Key>,
                RefCount,
            ),
        >,
    >,
    next_free: Option<Idx>,
    len: usize,
    _phantom: PhantomData<Id>,
}

pub struct StrongConfig<AllocCfg, Id: UniqueToken> {
    pub unique: Id,
    pub alloc: AllocCfg,
}

impl<Idx: IdxInt, S: Store, Alloc: AllocSelect, Id: UniqueToken, RefCount: IdxInt> Arena<S>
    for Strong<Idx, S, Alloc, Id, RefCount>
{
    type Cfg = StrongConfig<
        <Alloc::Impl<Idx, ValOrFree<Idx, (S::Data<Self::Key>, RefCount)>> as AllocImpl<
            Idx,
            ValOrFree<Idx, (S::Data<Self::Key>, RefCount)>,
        >>::Cfg,
        Id,
    >;
    type Key = Key<Idx, Id>;

    fn new(Self::Cfg { unique: _, alloc }: Self::Cfg) -> Self {
        Self {
            slots: Alloc::Impl::new(alloc),
            next_free: None,
            len: 0,
            _phantom: PhantomData,
        }
    }

    fn insert(&mut self, data: S::Data<Self::Key>) -> Option<Self::Key> {
        if let Some(idx) = self.next_free {
            unsafe {
                let slot = self.slots.write(idx);
                self.next_free = *slot.next_free;
                ManuallyDrop::drop(&mut slot.next_free);
                slot.data = ManuallyDrop::new((data, RefCount::ZERO));
            }
            self.len += 1;
            Some(Key::new(idx))
        } else if let Some(idx) = self.slots.insert(ValOrFree {
            data: ManuallyDrop::new((data, RefCount::ZERO)),
        }) {
            self.len += 1;
            Some(Key::new(idx))
        } else {
            None
        }
    }

    fn read(&self, key: &Self::Key) -> &S::Data<Self::Key> {
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

    fn len(&self) -> usize {
        self.len
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a <S as Store>::Data<Self::Key>> + 'a
    where
        <S as Store>::Data<<Self as Arena<S>>::Key>: 'a,
    {
        common::Iter::new(&self.slots, self.next_free, self.len()).map(|(data, _)| data)
    }
}

impl<Idx: IdxInt, S: Store, Alloc: AllocSelect, Id: UniqueToken, RefCount: IdxInt> DeleteArena<S>
    for Strong<Idx, S, Alloc, Id, RefCount>
{
    fn delete(&mut self, key: Self::Key) {
        unsafe {
            let entry = self.slots.write(key.0.idx);
            entry.data.1 = entry.data.1.dec();
            if entry.data.1 == RefCount::ZERO {
                ManuallyDrop::drop(&mut entry.data);
                entry.next_free = ManuallyDrop::new(self.next_free);
                self.next_free = Some(key.0.idx);
                self.len -= 1;
            }
        }
    }
}

impl<Idx: IdxInt, S: Store, Alloc: AllocSelect, Id: UniqueToken, RefCount: IdxInt> WriteArena<S>
    for Strong<Idx, S, Alloc, Id, RefCount>
{
    fn write(&mut self, key: &Self::Key) -> &mut S::Data<Self::Key> {
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

impl<Idx: IdxInt, S: Store, Alloc: AllocSelect, Id: UniqueToken, RefCount: IdxInt> CopyKeyArena<S>
    for Strong<Idx, S, Alloc, Id, RefCount>
{
    fn copy_key(&mut self, key: &<Self as Arena<S>>::Key) -> Option<<Self as Arena<S>>::Key> {
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
