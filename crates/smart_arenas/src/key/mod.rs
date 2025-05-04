// use crate::utils::idx::IdxInt;
use derive_where::derive_where;
use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    sync::{LazyLock, Mutex},
};
use vec_collections::{AbstractVecSet, VecSet};

mod idx;
pub use idx::IdxInt;

#[derive_where(PartialEq, Eq, Hash)]
pub struct Key<Unique: 'static, Idx: IdxInt> {
    pub(crate) idx: Idx,
    _phantom: PhantomData<Unique>,
}

impl<Unique: 'static, Idx: IdxInt> Drop for Key<Unique, Idx> {
    fn drop(&mut self) {
        if cfg!(debug_assertions) && !std::thread::panicking() {
            // JUSTIFY: Only panick when debug assertions is on
            //           - Leaking is safe, this check should have no cost in release (e.g.
            //             crashing in production), but provide benefit when testing
            // JUSTIFY: Panic at runtime when not in a panic itself.
            //           - Adding a const panic causes failures at compile time
            //             anywhere we might unwind. This causes issues when
            //             trying [ManuallyDrop::take], which can panic.
            //           Hence we settle for second best, a panic at runtime.
            //            - Has the additional benefit of producing a backtrace
            panic!(
                "Key<{}, {:?}> dropped without being disposed",
                std::any::type_name::<Unique>(),
                self.idx
            );
        }
    }
}

static USED_GUARDS: LazyLock<Mutex<VecSet<[std::any::TypeId; 10]>>> =
    LazyLock::new(|| Mutex::new(VecSet::empty()));
pub trait KeyTrait {
    type Unique: 'static;
    type Idx: IdxInt;

    unsafe fn to_key(idx: Self::Idx) -> Self;
    fn to_idx(&self) -> Self::Idx;
    fn dispose(self);
    fn guard() {
        let mut l = USED_GUARDS.lock().unwrap();
        let id = std::any::TypeId::of::<Self::Unique>();
        if l.contains(&id) {
            panic!(
                "Attempted to use type {} to construct a unique more than once",
                std::any::type_name::<Self::Unique>()
            );
        } else {
            l.insert(id);
        }
    }

    /// # Safety
    /// Should only be called once there is a guarentee that no keys exist for the guard.
    ///  - Enforced by the arena that uses it.
    unsafe fn relinquish() {
        let mut l = USED_GUARDS.lock().unwrap();
        let id = std::any::TypeId::of::<Self::Unique>();
        if !l.remove(&id) {
            panic!(
                "Cannot relinquish guard if guard was never taken for {}",
                std::any::type_name::<Self::Unique>()
            );
        }
    }
}

impl<Unique: 'static, Idx: IdxInt> KeyTrait for Key<Unique, Idx> {
    type Unique = Unique;
    type Idx = Idx;

    unsafe fn to_key(idx: Self::Idx) -> Self {
        Self {
            idx,
            _phantom: PhantomData,
        }
    }

    fn to_idx(&self) -> Self::Idx {
        self.idx
    }

    fn dispose(self) {
        std::mem::forget(self);
    }
}

pub struct WeakKey<'arena, Key: KeyTrait> {
    key: ManuallyDrop<Key>,
    _phantom: PhantomData<&'arena ()>,
}

impl<'arena, Key: KeyTrait> WeakKey<'arena, Key> {
    pub unsafe fn new_from_idx(idx: Key::Idx) -> Self {
        Self {
            key: ManuallyDrop::new(unsafe { Key::to_key(idx) }),
            _phantom: PhantomData,
        }
    }

    pub fn brw(&self) -> &'_ Key {
        &self.key
    }
}

#[macro_export]
macro_rules! define_keys {
    (  $( $name:ident => $ty:ty ),* $(,)? ) => {
        use $crate::key::Key;
        pub mod tags {
            $( pub struct $name; )*
        }
        $( pub type $name = Key<tags::$name, $ty>; )*
    };
}
