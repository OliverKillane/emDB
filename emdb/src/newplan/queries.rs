use std::collections::HashMap;
use proc_macro2::Ident;
use super::{ScalarType, Key, Operator};

pub struct Query {
    pub name: Ident,
    pub params: HashMap<Ident, Key<ScalarType>>,
    
    /// INV is a [LogicalOp::Return]
    pub returnval: Option<Key<Operator>>,
}