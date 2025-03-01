use std::hash::Hash;

pub mod own;
pub mod strong;

pub trait Arena<Data> {
    type Cfg;
    type Key: Hash + Eq;

    fn new(cfg: Self::Cfg) -> Self;
    fn insert(&mut self, data: Data) -> Option<Self::Key>;

    fn read(&self, key: &Self::Key) -> &Data;
    
    fn delete(&mut self, key: Self::Key);
}

pub trait WriteArena<Data>: Arena<Data> {
    fn write(&mut self, key: &Self::Key) -> &mut Data;
}

pub trait CopyKeyArena<Data>: Arena<Data> {
    fn copy_key(&mut self, key: &<Self as Arena<Data>>::Key) -> Option<<Self as Arena<Data>>::Key>;
}

mod common {
    use derive_where::derive_where;

    use crate::utils::{
        drop::{CanDropWith, DropWith},
        idx::IdxInt,
        unique::UniqueToken,
    };
    use std::{marker::PhantomData, mem::ManuallyDrop};

    pub struct Cfg<AllocCfg, Id: UniqueToken> {
        pub unique: Id,
        pub alloc: AllocCfg,
    }

    #[derive_where(Hash, PartialEq, Eq)]
    pub struct KeyInner<Idx: IdxInt, Id: UniqueToken> {
        pub idx: Idx,
        _phantom: PhantomData<Id>,
    }

    impl<Idx: IdxInt, Id: UniqueToken> CanDropWith<()> for KeyInner<Idx, Id> {
        fn drop(self, _: ()) {}
    }

    #[derive_where(Hash, PartialEq, Eq)]
    pub struct Key<Idx: IdxInt, Id: UniqueToken>(pub DropWith<KeyInner<Idx, Id>>);

    impl<Idx: IdxInt, Id: UniqueToken> Key<Idx, Id> {
        pub fn new(idx: Idx) -> Self {
            Self(DropWith::new(KeyInner {
                idx,
                _phantom: PhantomData,
            }))
        }
    }

    pub union ValOrFree<Idx: IdxInt, Data> {
        pub data: ManuallyDrop<Data>,
        pub next_free: ManuallyDrop<Option<Idx>>,
    }
}
