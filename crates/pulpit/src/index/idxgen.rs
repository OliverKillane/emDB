//! # Generational Indices
//! Supports use of generational indices to track deletions.

use super::{Index, IndexPull};

pub struct IdxGen {}

impl Index for IdxGen {
    type ExternalIndex = (usize, usize);
    type InitData = ();
    type NewIndexError = ();

    fn new(init: Self::InitData) -> Self {
        todo!()
    }
    
    fn to_index(&self, index: Self::ExternalIndex) -> Result<crate::column::ColInd, super::IncorrectIndexError> {
        todo!()
    }
    
    fn new_index(&mut self) -> Result<Self::ExternalIndex, Self::NewIndexError> {
        todo!()
    }
    
    fn count(&self) -> usize {
        todo!()
    }    
}

impl IndexPull for IdxGen {
    fn pull_index(&mut self, index: Self::ExternalIndex) -> Result<crate::column::ColInd, super::IncorrectIndexError> {
        todo!()
    }
}
