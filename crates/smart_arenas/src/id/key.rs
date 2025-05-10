use std::marker::{PhantomCovariantLifetime, PhantomInvariantLifetime};

use super::index::Index;

/// A trait for key types.
///  - A typed wrapper around an [crate::id::index::Index], that binds the type of the key to the
///    [crate::arena::Arena].
pub unsafe trait KeyTrait<'id> {
    type Idx: Index;

    unsafe fn to_key(idx: Self::Idx) -> Self;
    fn to_idx(&self) -> Self::Idx;
}

pub struct Key<'id, Idx: Index> {
    idx: Idx,
    _phantom: PhantomInvariantLifetime<'id>,
}

unsafe impl<'id, Idx: Index> KeyTrait<'id> for Key<'id, Idx> {
    type Idx = Idx;

    unsafe fn to_key(idx: Self::Idx) -> Self {
        Self {
            idx,
            _phantom: PhantomInvariantLifetime::new(),
        }
    }

    fn to_idx(&self) -> Self::Idx {
        self.idx
    }
}

pub struct WeakKey<'id, 'brw, Key: KeyTrait<'id>> {
    idx: Key::Idx,
    _phantom_id: PhantomInvariantLifetime<'id>,
    _phantom_brw: PhantomCovariantLifetime<'brw>,
}

impl<'id, 'brw, Key: KeyTrait<'id>> WeakKey<'id, 'brw, Key> {
    pub unsafe fn to_key(idx: Key::Idx) -> Self {
        Self {
            idx,
            _phantom_id: PhantomInvariantLifetime::new(),
            _phantom_brw: PhantomCovariantLifetime::new(),
        }
    }

    pub unsafe fn to_idx(&self) -> Key::Idx {
        self.idx
    }
}
