//! Indexes for gathering values from a group of columns.

use crate::column::ColInd;

mod idxgen;
pub use idxgen::*;

mod idxpush;
pub use idxpush::*;

pub struct IncorrectKeyError;

pub trait Index {
    type Key: Copy + Eq;
    type InitData;

    fn new(init: Self::InitData) -> Self;

    fn to_index(&self, key: Self::Key) -> Result<ColInd, IncorrectKeyError>;
    fn new_index(&mut self) -> Self::Key;

    fn count(&self) -> usize;
}

pub trait IndexPull: Index {
    fn pull_index(&mut self, key: Self::Key) -> Result<ColInd, IncorrectKeyError>;
}

mod verif {
    use super::*;
    use std::collections::HashMap;

    struct ReferenceIdx<Idx: Index> {
        idx: Idx,
        set: HashMap<Idx::Key, usize>,
    }

    impl<Idx: Index> ReferenceIdx<Idx>
    where
        Idx::Key: std::hash::Hash,
    {
        fn new(init: Idx::InitData) -> Self {
            ReferenceIdx {
                idx: Idx::new(init),
                set: HashMap::new(),
            }
        }

        fn check_to_index(&self, key: Idx::Key) {
            if let Some(ind) = self.set.get(&key) {
                if let Ok(got_ind) = self.idx.to_index(key) {
                    assert_eq!(*ind, got_ind, "Index returned the wrong index for a key");
                } else {
                    panic!("Index incorrectly rejected a key");
                }
            } else {
                assert!(
                    self.idx.to_index(key).is_err(),
                    "Index incorrectly accepted a key"
                );
            }
        }

        fn check_new_index(&mut self) -> Idx::Key {
            let new_ind = self.idx.new_index();
            assert_eq!(
                self.set.insert(new_ind, self.set.len()),
                None,
                "Created index that is already present"
            );
            new_ind
        }

        fn check_count(&self) {
            assert_eq!(self.set.len(), self.idx.count(), "Index incorrect count");
        }
    }

    impl<Idx: IndexPull> ReferenceIdx<Idx>
    where
        Idx::Key: std::hash::Hash,
    {
        fn check_pull_index(&mut self, key: Idx::Key) {
            if let Some(ind) = self.set.remove(&key) {
                if let Ok(got_ind) = self.idx.pull_index(key) {
                    assert_eq!(ind, got_ind, "Index returned the wrong index for a key");
                } else {
                    panic!("Index incorrectly rejected a key");
                }
            } else {
                assert!(
                    self.idx.pull_index(key).is_err(),
                    "Index incorrectly pulled a nonexistent key"
                );
            }
        }
    }

    #[cfg(test)]
    mod test_verif {
        use super::*;

        #[test]
        fn test_index_push() {
            let mut idx = ReferenceIdx::<IndexPush>::new(());
            idx.check_to_index(0);
            idx.check_new_index();
            idx.check_count();
        }
    }
}
