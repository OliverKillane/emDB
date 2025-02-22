use std::marker::PhantomData;

use crate::{ints::IdxInt, slots::Slots, unique::Unique};

use super::Arena;

struct Own<Idx: IdxInt, Data, Slot: Slots<Idx, Data>> {
    slots: Slot,
    _phantom: PhantomData<(Idx, Data)>
}

struct Key<Idx> {
    idx: Idx
}

impl <Idx: IdxInt, Data, Slot: Slots<Idx, Data>> Arena<Data> for Own<Idx, Data, Slot> {
    type Cfg = ();
    type Key = Key<Idx>;

    fn new(cfg: Self::Cfg) -> Self {
        todo!()
    }

    fn insert(&mut self, data: Data) -> Self::Key {
        todo!()
    }

    fn read(&self, idx: &Self::Key) -> &Data {
        todo!()
    }

    fn write(&self, idx: &Self::Key) -> &mut Data {
        todo!()
    }

    fn delete(&mut self, idx: Self::Key) {
        todo!()
    }
}

