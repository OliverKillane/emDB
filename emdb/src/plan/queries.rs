use super::{Key, Operator, Plan, ScalarType};
use proc_macro2::Ident;
use std::collections::HashMap;

pub struct Query {
    pub name: Ident,
    pub params: HashMap<Ident, Key<ScalarType>>,

    /// INV is a [super::FlowOperator::Return]
    pub returnval: Option<Key<Operator>>,
}

impl Plan {
    pub fn get_query(&self, key: Key<Query>) -> &Query {
        self.queries.get(key).unwrap()
    }
}
