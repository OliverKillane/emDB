//! # An append-only index
//! Does not support deletions
use super::Index;

pub struct IndexPush {
    size: usize,
}

impl Index for IndexPush {
    type Key = usize;
    type InitData = ();

    fn new(init: Self::InitData) -> Self {
        IndexPush { size: 0 }
    }

    fn to_index(
        &self,
        index: Self::Key,
    ) -> Result<crate::column::ColInd, super::IncorrectKeyError> {
        if index < self.size {
            Ok(index)
        } else {
            Err(super::IncorrectKeyError)
        }
    }

    fn count(&self) -> usize {
        self.size
    }

    fn new_index(&mut self) -> Self::Key {
        self.size += 1;
        self.size - 1
    }
}
