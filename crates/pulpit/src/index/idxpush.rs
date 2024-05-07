//! # An append-only index
//! Does not support deletions
use super::{Index, IndexPull};

pub struct IndexPush {
    size: usize
}

impl Index for IndexPush {
    type ExternalIndex = usize;
    type InitData = usize;

    fn new(init: Self::InitData) -> Self {
        IndexPush { size: init }
    }
    
    fn to_index(&self, index: Self::ExternalIndex) -> Result<crate::column::ColInd, super::IncorrectIndexError> {
        if index < self.size {
            Ok(index)
        } else {
            Err(super::IncorrectIndexError)
        }
    }
    
    fn new_index(&mut self) -> Result<Self::ExternalIndex, crate::column::AllocationFailure> {
        self.size += 1;
        Ok(self.size - 1)
    }
}