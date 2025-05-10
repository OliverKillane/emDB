use crate::id::{
    key::{KeyTrait, WeakKey},
    token::Token,
};

mod app;
mod bind;
mod own;
mod share;

pub use app::*;
pub use bind::*;
pub use own::*;
pub use share::*;

/// ## A basic read only Arena
/// An arena data structure, storing values associated with a container determined key.
/// - Capabilities of arenas are expressed through traits (for example [DeleteArena])
/// - All arenas use keys with the same [KeyTrait], separating the key type from the
///   arena allows for arenas containing complex self referential keys, or cyclic (between
///   arenas) keys.
pub trait Arena<'id> {
    type Key: KeyTrait<'id>;
    type Data;
    type Read<'a>
    where
        Self: 'a;

    fn new(preallocate_to: <Self::Key as KeyTrait<'id>>::Idx, token: Token<'id>) -> Self;
    fn insert_return_reuse(&mut self, data: Self::Data) -> Option<(Self::Key, bool)>;
    fn insert(&mut self, data: Self::Data) -> Option<Self::Key> {
        self.insert_return_reuse(data).map(|(k, _)| k)
    }
    fn read(&self, key: &Self::Key) -> Self::Read<'_>;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> impl Iterator<Item = Self::Read<'_>> + '_ {
        self.iter_with_weak_key().map(|(_, data)| data)
    }

    fn iter_with_weak_key<'a>(
        &'a self,
    ) -> impl Iterator<Item = (WeakKey<'id, 'a, Self::Key>, Self::Read<'a>)> + 'a;
}

/// An arena supporting deletion of keys.
pub trait DeleteArena<'id>: Arena<'id> {
    fn delete_return_dropped(&mut self, key: Self::Key) -> bool;
    fn delete(&mut self, key: Self::Key) {
        _ = self.delete_return_dropped(key);
    }
}

/// An arena supporting mutable values.
pub trait WriteArena<'id>: Arena<'id> {
    type Write<'a>
    where
        Self: 'a;
    fn write(&mut self, key: &Self::Key) -> Self::Write<'_>;
}

/// An arena that allows keys to be copied.
pub trait CopyKeyArena<'id>: Arena<'id> {
    fn copy_key(&mut self, key: &<Self as Arena<'id>>::Key) -> Option<<Self as Arena<'id>>::Key>;
}

/// Transform all members of an arena, consuming the arena, but leaving the keys as valid.
// TODO: Implement this trait, with in-place specialisation
pub trait TransformArena<'id, InputData, OutputData>: Arena<'id, Data = InputData> {
    fn transform(
        self,
        f: impl Fn(InputData) -> OutputData,
    ) -> impl Arena<'id, Data = OutputData, Key = Self::Key>;
}

pub trait IterKeyArena<'id>: Arena<'id> {
    fn iter_with_key(&self) -> impl Iterator<Item = (Self::Key, Self::Read<'_>)> + '_;
}

mod common {
    use crate::id::index::Index;
    use std::mem::ManuallyDrop;

    pub union ValOrFree<Idx: Index, Data> {
        pub data: ManuallyDrop<Data>,
        pub next_free: ManuallyDrop<Option<Idx>>,
    }
}
