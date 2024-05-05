//! Indexes for gathering values from a group of columns.

trait Index {
    type ExternalInd;
    type InternalInd;
    fn lookup(&self, ind: Self::ExternalInd) -> Self::InternalInd;
}