use super::Arena;
use crate::{
    alloc::{AllocImpl, AllocSelect},
    utils::{
        drop::{CanDropWith, DropWith},
        idx::IdxInt,
        unique::Unique,
    },
};
use std::{marker::PhantomData, mem::ManuallyDrop};

struct Own<Idx: IdxInt, Data, Alloc: AllocSelect, Id: Unique> {
    slots: Alloc::Impl<Idx, Value<Idx, Data>>,
    next_free: Option<Idx>,
    _phantom: PhantomData<Id>,
}

pub struct Cfg<AllocCfg, Id: Unique> {
    unique: Id,
    alloc: AllocCfg,
}

struct KeyInner<Idx, Id: Unique> {
    idx: Idx,
    _phantom: PhantomData<Id>,
}

impl<Idx, Id: Unique> CanDropWith<()> for KeyInner<Idx, Id> {
    fn drop(self, _: ()) {}
}

pub struct Key<Idx, Id: Unique>(DropWith<KeyInner<Idx, Id>>);

impl<Idx, Id: Unique> Key<Idx, Id> {
    fn new(idx: Idx) -> Self {
        Self(DropWith::new(KeyInner {
            idx,
            _phantom: PhantomData,
        }))
    }
}

union Value<Idx: IdxInt, Data> {
    data: ManuallyDrop<Data>,
    next_free: ManuallyDrop<Option<Idx>>,
}

impl<Idx: IdxInt, Data, Alloc: AllocSelect, Id: Unique> Arena<Data> for Own<Idx, Data, Alloc, Id> {
    type Cfg =
        Cfg<<Alloc::Impl<Idx, Value<Idx, Data>> as AllocImpl<Idx, Value<Idx, Data>>>::Cfg, Id>;
    type Key = Key<Idx, Id>;

    fn new(Self::Cfg { unique, alloc }: Self::Cfg) -> Self {
        unique.runtime_check();
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
                .insert(Value {
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

#[cfg(test)]
mod tests {
    use crate::{
        alloc::contig::{Contig, ContigCfg},
        unique,
    };

    use super::*;

    #[test]
    #[should_panic]
    fn check_panic_on_leak() {
        #[derive(Clone, PartialEq, Eq, Debug)]
        struct MyData {
            foo: i32,
        }

        let data = MyData { foo: 23 };

        let mut arena = Own::<u16, MyData, Contig, _>::new(Cfg {
            unique: unique!(),
            alloc: ContigCfg {
                preallocate_to: 123,
            },
        });
        let key = arena.insert(data.clone()).unwrap();
        assert_eq!(&data, arena.read(&key));
        // key is dropped wthout deletion
    }
}
