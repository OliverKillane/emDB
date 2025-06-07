use std::marker::{PhantomCovariantLifetime, PhantomInvariantLifetime};

use super::index::Index;

/// A trait for key types.
///  - A typed wrapper around an [crate::id::index::Index], that binds the type of the key to the
///    [crate::arena::Arena].
///
/// # Safety
/// Safety of arena accesses rely on no safe method for copying a key existing.
///  - Either by clone, copy, or some other method.
///  - If able to safely clone, reference counts in [crate::arena::Share], and single ownership of
///    [crate::arena::Own] will no longer hold.
pub unsafe trait KeyTrait<'id> {
    type Idx: Index;

    /// # Safety
    /// Only used from within arenas, and only where a key copy does not break any arena internal invariants.
    unsafe fn from_idx(idx: Self::Idx) -> Self;
    fn to_idx(&self) -> Self::Idx;
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Key<'id, Idx: Index> {
    idx: Idx,
    _phantom: PhantomInvariantLifetime<'id>,
}

unsafe impl<'id, Idx: Index> KeyTrait<'id> for Key<'id, Idx> {
    type Idx = Idx;

    /// # Safety
    /// Only used from within arenas, and only where a key copy does not break any arena internal invariants.
    unsafe fn from_idx(idx: Self::Idx) -> Self {
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

impl<'id, Key: KeyTrait<'id>> WeakKey<'id, '_, Key> {
    /// # Safety
    /// Only used from within arenas, and only where a key copy does not break any arena internal invariants.
    pub unsafe fn from_idx(idx: Key::Idx) -> Self {
        Self {
            idx,
            _phantom_id: PhantomInvariantLifetime::new(),
            _phantom_brw: PhantomCovariantLifetime::new(),
        }
    }

    pub fn to_idx(&self) -> Key::Idx {
        self.idx
    }
}
