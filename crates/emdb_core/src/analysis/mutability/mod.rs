//! ## Geting the tables mutated by queries
//! This code is the same as for the Serialized
use crate::plan;

#[enumtrait::store(trait_get_muts)]
pub trait GetMuts {
    /// Update the set with the mutated tables
    fn mutates(&self, lp: & plan::Plan) -> bool {
        false
    }
}

impl GetMuts for plan::Query {
    fn mutates(&self, lp: & plan::Plan) -> bool {
        lp.get_context(self.ctx).mutates(lp)
    }
}

impl GetMuts for plan::Context {
    fn mutates(&self, lp: & plan::Plan) -> bool {
        self.ordering
            .iter()
            .map(|k| lp.get_operator(*k).mutates(lp))
            .any(|x| x)
    }
}

#[enumtrait::impl_trait(trait_get_muts for plan::operator_enum)]
impl GetMuts for plan::Operator {}

impl GetMuts for plan::Insert {
    fn mutates(&self, lp: & plan::Plan) -> bool {
        true
    }
}
impl GetMuts for plan::Update {
    fn mutates(&self, lp: & plan::Plan) -> bool {
        true
    }
}
impl GetMuts for plan::Delete {
    fn mutates(&self, lp: & plan::Plan) -> bool {
        true
    }
}

impl GetMuts for plan::GroupBy {
    fn mutates(&self, lp: & plan::Plan) -> bool {
        lp.get_context(self.inner_ctx).mutates(lp)
    }
}
impl GetMuts for plan::ForEach {
    fn mutates(&self, lp: & plan::Plan) -> bool {
        lp.get_context(self.inner_ctx).mutates(lp)
    }
}

impl GetMuts for plan::UniqueRef {}
impl GetMuts for plan::ScanRefs {}
impl GetMuts for plan::DeRef {}
impl GetMuts for plan::Map {}
impl GetMuts for plan::Expand {}
impl GetMuts for plan::Fold {}
impl GetMuts for plan::Filter {}
impl GetMuts for plan::Sort {}
impl GetMuts for plan::Assert {}
impl GetMuts for plan::Take {}
impl GetMuts for plan::Collect {}
impl GetMuts for plan::Join {}
impl GetMuts for plan::Fork {}
impl GetMuts for plan::Union {}
impl GetMuts for plan::Row {}
impl GetMuts for plan::Return {}
impl GetMuts for plan::Discard {}
