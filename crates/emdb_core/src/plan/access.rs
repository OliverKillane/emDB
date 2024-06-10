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

impl<'imm, T> Clone for Idx<'imm, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'imm, T> Copy for Idx<'imm, T> {}
impl<'imm, T> PartialEq for Idx<'imm, T> {
    fn eq(&self, other: &Self) -> bool {
        self.arr_idx == other.arr_idx
    }
}
impl<'imm, T> Eq for Idx<'imm, T> {}
impl<'imm, T> std::hash::Hash for Idx<'imm, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.arr_idx.hash(state)
    }
}
impl<'imm, T> std::ops::Deref for Idx<'imm, T> {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.arr_idx
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

impl<'imm, T> Clone for ImmKey<'imm, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'imm, T> Copy for ImmKey<'imm, T> {}
impl<'imm, T> PartialEq for ImmKey<'imm, T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<'imm, T> Eq for ImmKey<'imm, T> {}
impl<'imm, T> std::hash::Hash for ImmKey<'imm, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.arr_idx().hash(state)
    }
}
impl<'imm, T> std::ops::Deref for ImmKey<'imm, T> {
    type Target = Key<T>;
    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

/// A wrapper type for implementing traits on components that need to use the
/// plan for context.
/// - for example printing types requires the logical plan for table ref types
pub struct With<'a, A> {
    pub plan: &'a Plan,
    pub extended: A,
}
