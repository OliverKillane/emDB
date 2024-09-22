//! ## Scope mutability tracking
//! For closures we need to track mutability of different tables and fields.
//!
//! ```text
//! |&mut A, &B, &C, &D| {
//!     
//!     &A
//!
//!     |&B, &mut A| {
//!         &B
//!         &mut A
//!     }
//!
//!     |&C, &D| {
//!         &C
//!         &D
//!     }
//! }
//! ```

use itertools::Itertools;
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Mut,
    Imm,
}

/// A handle for a stack of scopes, that propagates usages upwards.
// NOTE: Implemented using `Rc<RefCell<_>>`` to avoid complication of containing
//       borrowing of same type.
//
//       Attempt 1:
//       ```
//       enum ClosureMuts<'parent, Key> {
//           ..  parent: &'parent mut ClosureMuts<'parent, Key>,
//       }
//       ```
//       which results in a problem as both the borrow and used lifetime are the
//       same. We need to take a borrow `'sub` of the parent, and have `&'sub mut`
//       pointing to the original `ClosureMut<'parent, Key>`. However because we
//       only used one lifetime in the definition of `ClosureMut` we must have
//       `&'sub mut ClosureMut<'parent, Key>` converted to `&'sub mut ClosureMut<'sub, Key>`
//
//       Unfortunately `&'a mut T` is invariant in T, meaning so subtyping
//       relation is present, so no conversion can occur.
//
//       Therefore this does not work.
//
//       Attempt 2:
//       ```
//       enum ClosureMuts<'sub, 'parent: 'sub, Key> {
//           ..  parent: &'sub mut ClosureMuts<'parent, ????? , Key>,
//       }
//       ```
//       Now we just need another lifetime param, which then requires another
//       lifetime param, and so on.
//
//       This is annoying.
//
//       Attempt 3:
//       Abuse `impl trait` somehow to hide extra lifetime parameters.
//       - Works without recursion, once recursive you get infinite types.
#[derive(Debug)]
pub struct ScopeHandle<'parent, Key> {
    parent: &'parent mut ScopeData<Key>,
    depth: usize,
}

#[derive(Debug)]
pub struct ScopeData<Key> {
    data: Vec<HashMap<Key, Mutability>>,
}

impl<Key: Eq + Hash + Clone + Ord> ScopeData<Key> {
    pub fn new() -> Self {
        ScopeData { data: Vec::new() }
    }

    pub fn scope(&mut self) -> ScopeHandle<'_, Key> {
        ScopeHandle::from_data(self)
    }
}

fn add_mut<Key: Eq + Hash>(muts: &mut HashMap<Key, Mutability>, key: Key) -> bool {
    match muts.get_mut(&key) {
        Some(Mutability::Mut) => false,
        Some(Mutability::Imm) => {
            muts.insert(key, Mutability::Mut);
            true
        }
        None => {
            muts.insert(key, Mutability::Mut);
            true
        }
    }
}

fn add_imm<Key: Eq + Hash>(muts: &mut HashMap<Key, Mutability>, key: Key) -> bool {
    match muts.get_mut(&key) {
        Some(_) => false,
        None => {
            muts.insert(key, Mutability::Imm);
            true
        }
    }
}

impl<'parent, Key> ScopeHandle<'parent, Key>
where
    Key: Eq + Hash + Clone + Ord,
{
    pub fn from_data(parent: &mut ScopeData<Key>) -> ScopeHandle<'_, Key> {
        let depth = parent.data.len();
        parent.data.push(HashMap::new());
        ScopeHandle { depth, parent }
    }

    // Returns an iterator with consistent ordering (by key value)
    pub fn mutabilities(&self) -> impl Iterator<Item = (&Key, &Mutability)> {
        // NOTE: Because we need consistent ordering for this method's use in
        //       closures (where the args, and params definitions must be the
        //       same order), we sort here.
        self.parent.data[self.depth]
            .iter()
            .sorted_by_key(|(k, _)| *k)
    }

    pub fn mutates(&self) -> bool {
        self.parent.data[self.depth]
            .values()
            .any(|m| *m == Mutability::Mut)
    }

    pub fn is_empty(&self) -> bool {
        self.parent.data[self.depth].is_empty()
    }

    pub fn scope(&mut self) -> ScopeHandle<'_, Key> {
        Self::from_data(self.parent)
    }

    pub fn add_mut(&mut self, key: Key) {
        let mut index: usize = self.depth;
        while add_mut(&mut self.parent.data[index], key.clone()) {
            if index == 0 {
                break;
            }
            index -= 1;
        }
    }
    pub fn add_imm(&mut self, key: Key) {
        let mut index: usize = self.depth;
        while add_imm(&mut self.parent.data[index], key.clone()) {
            if index == 0 {
                break;
            }
            index -= 1;
        }
    }
}

impl<'parent, Key> Drop for ScopeHandle<'parent, Key> {
    // INV: we rely on the borrow used for child-scope ensuring drops are called from the lowest
    //      scope, upwards. Hence we can just pop the end of the vector as we ascend.
    fn drop(&mut self) {
        assert_eq!(self.depth, self.parent.data.len() - 1); // TODO: remove potential panic from inside a `drop`
        self.parent.data.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_muts<Key: Clone + Hash + Eq + Ord + std::fmt::Debug>(
        muts: &ScopeHandle<'_, Key>,
        expected: Vec<(&Key, &Mutability)>,
    ) {
        let mut actual = muts
            .mutabilities()
            .sorted_by_key(|(k, _)| *k)
            .collect::<Vec<_>>();
        actual.sort_by_key(|(k, _)| *k);
        assert_eq!(actual, expected);
    }

    #[test]
    fn passing_scopes() {
        fn internal<'brw, 'parent: 'brw>(
            scope: &'brw mut ScopeHandle<'parent, i32>,
        ) -> ScopeHandle<'brw, i32> {
            let mut x = scope.scope();
            x.add_imm(0);
            x
        }

        let mut data = ScopeData::new();
        let mut parent = data.scope();

        {
            let mut child = internal(&mut parent);
            child.add_mut(1);
            assert_muts(&child, vec![(&0, &Mutability::Imm), (&1, &Mutability::Mut)]);
            let new_child = internal(&mut child);
            assert_muts(&new_child, vec![(&0, &Mutability::Imm)]);
        }
        assert_muts(
            &parent,
            vec![(&0, &Mutability::Imm), (&1, &Mutability::Mut)],
        );
    }

    #[test]
    fn basic() {
        let mut data = ScopeData::new();
        let mut parent = data.scope();
        parent.add_imm(1);

        {
            let mut child = parent.scope();
            child.add_mut(2);
        }
        assert_muts(
            &parent,
            vec![(&1, &Mutability::Imm), (&2, &Mutability::Mut)],
        );
    }

    #[test]
    fn upgrade() {
        let mut data = ScopeData::new();
        let mut parent = data.scope();
        parent.add_imm(1);

        {
            let mut child = parent.scope();
            child.add_mut(1);
        }
        assert_muts(&parent, vec![(&1, &Mutability::Mut)]);
    }

    #[test]
    fn nochange() {
        let mut data = ScopeData::new();
        let mut parent = data.scope();
        parent.add_mut(1);

        {
            let mut child = parent.scope();
            child.add_imm(1);
            assert_muts(&child, vec![(&1, &Mutability::Imm)]);
        }
        assert_muts(&parent, vec![(&1, &Mutability::Mut)]);
    }

    #[test]
    fn scopes() {
        let mut data = ScopeData::new();
        let mut parent = data.scope();
        let mut child = parent.scope();

        assert_muts(&child, vec![]);

        child.add_imm(9);

        assert_muts(&child, vec![(&9, &Mutability::Imm)]);

        {
            let mut child2 = child.scope();
            child2.add_mut(10);

            assert_muts(&child2, vec![(&10, &Mutability::Mut)]);

            {
                let mut child3 = child2.scope();
                child3.add_mut(11);

                assert_muts(&child3, vec![(&11, &Mutability::Mut)]);
            }

            assert_muts(
                &child2,
                vec![(&10, &Mutability::Mut), (&11, &Mutability::Mut)],
            );

            {
                let mut child4 = child2.scope();
                child4.add_mut(14);

                assert_muts(&child4, vec![(&14, &Mutability::Mut)]);
            }

            assert_muts(
                &child2,
                vec![
                    (&10, &Mutability::Mut),
                    (&11, &Mutability::Mut),
                    (&14, &Mutability::Mut),
                ],
            );
        }
    }
}
