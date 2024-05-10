//! # Generational Indices
//! Supports use of generational indices to track deletions.

use super::{Index, IndexPull};

// TODO: Complete

pub struct IdxGen {
    next_free: Option<usize>,
    count: usize,
    items: Vec<usize>,
}

impl Index for IdxGen {
    type Key = (usize, usize);
    type InitData = ();

    fn new(init: Self::InitData) -> Self {
        todo!()
    }

    fn to_index(
        &self,
        index: Self::Key,
    ) -> Result<crate::column::ColInd, super::IncorrectKeyError> {
        todo!()
    }

    fn new_index(&mut self) -> Self::Key {
        todo!()
    }

    fn count(&self) -> usize {
        todo!()
    }
}

impl IndexPull for IdxGen {
    fn pull_index(
        &mut self,
        index: Self::Key,
    ) -> Result<crate::column::ColInd, super::IncorrectKeyError> {
        todo!()
    }
}
