//! Helpers for accessing [`Plan`] values through keys, and indexes to immutable plans

use super::*;

/// All component types can be indexed through a [Key]
/// - No shared mutability, need to have the plan also to use
/// - Checked access for keys to ensure no use after delete
/// - Keys are generational, so no aliasing of old deleted, vs new keys is
///   possible.
pub type Key<T> = Index<T, usize, NonzeroGeneration<usize>>;

/// When a key into an immutable plan is needed, but the plan is not changed:
/// - Can use a borrow to enforce the plan is not mutated
/// - Can ignore the generation count on keys
///
/// It is a zero cost wrapper (no extra memory used, exists only to supplement
/// usize in type checking).
/// ```ignore
/// # fn wrapper<'imm, T>() {
/// assert_eq!(std::mem::size_of::<usize>(), std::mem::size_of::<Idx<'imm, T>>());
/// # }
/// ```
// TODO: Make this doctest runnable
pub struct Idx<'imm, T> {
    arr_idx: usize,
    plan_holder: &'imm (),
    _phantom: std::marker::PhantomData<T>,
}

impl<'imm, T> Idx<'imm, T> {
    pub fn new(key: Key<T>, plan: &'imm Plan) -> Self {
        Idx {
            arr_idx: key.arr_idx(),
            plan_holder: &plan._holder,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'imm, T> From<(Key<T>, &'imm Plan)> for Idx<'imm, T> {
    fn from((key, plan): (Key<T>, &'imm Plan)) -> Self {
        Self::new(key, plan)
    }
}

impl<T> Clone for Idx<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Idx<'_, T> {}
impl<T> PartialEq for Idx<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.arr_idx == other.arr_idx
    }
}
impl<T> Eq for Idx<'_, T> {}
impl<T> std::hash::Hash for Idx<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.arr_idx.hash(state)
    }
}
impl<T> std::ops::Deref for Idx<'_, T> {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.arr_idx
    }
}

impl<T> PartialOrd for Idx<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Idx<'_, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.arr_idx.cmp(&other.arr_idx)
    }
}

/// A key with a lifetime binding to prevent mutation of the referenced plan.
/// - Implements hash (unlike [`typed_generational_arena::Index`] (the generation count is not hashable))
///   TODO: contribute to [`typed_generational_arena::Index`] to fix this.
pub struct ImmKey<'imm, T> {
    key: Key<T>,
    plan_holder: &'imm (),
}
impl<'imm, T> ImmKey<'imm, T> {
    pub fn new(key: Key<T>, plan: &'imm Plan) -> Self {
        Self {
            key,
            plan_holder: &plan._holder,
        }
    }
}

impl<'imm, T> From<(Key<T>, &'imm Plan)> for ImmKey<'imm, T> {
    fn from((key, plan): (Key<T>, &'imm Plan)) -> Self {
        Self::new(key, plan)
    }
}

impl<T> Clone for ImmKey<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for ImmKey<'_, T> {}
impl<T> PartialEq for ImmKey<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<T> Eq for ImmKey<'_, T> {}
impl<T> std::hash::Hash for ImmKey<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.arr_idx().hash(state)
    }
}
impl<T> std::ops::Deref for ImmKey<'_, T> {
    type Target = Key<T>;
    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

impl<T> PartialOrd for ImmKey<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for ImmKey<'_, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.arr_idx().cmp(&other.arr_idx())
    }
}

/// A wrapper type for implementing traits on components that need to use the
/// plan for context.
/// - for example printing types requires the logical plan for table ref types
pub struct With<'a, A> {
    pub plan: &'a Plan,
    pub extended: A,
}
