use super::{Key, Operator, ScalarType};
use proc_macro2::Ident;
use std::collections::HashMap;

pub struct Query {
    pub name: Ident,
    pub params: HashMap<Ident, Key<ScalarType>>,

    /// INV is a [LogicalOp::Return]
    pub returnval: Option<Key<Operator>>,
}
