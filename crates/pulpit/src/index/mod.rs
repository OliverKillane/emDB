//! Indexes for gathering values from a group of columns.

use crate::column::{ColInd};

mod idxgen; pub use idxgen::*;

struct IncorrectIndexError;

trait Index {
    type ExternalIndex: Copy;
    type InitData;
    type NewIndexError;

    fn new(init: Self::InitData) -> Self;

    fn to_index(&self, index: Self::ExternalIndex) -> Result<ColInd, IncorrectIndexError>;
    fn new_index(&mut self) -> Result<Self::ExternalIndex, Self::NewIndexError>;

    fn count(&self) -> usize;
}

trait IndexPull: Index {
    fn pull_index(&mut self, index: Self::ExternalIndex) -> Result<ColInd, IncorrectIndexError>;
}
