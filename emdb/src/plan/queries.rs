use super::{Key, Operator, Plan, ScalarType};
use proc_macro2::Ident;

pub struct Query {
    pub name: Ident,
    pub ctx: Key<Context>
}

/// A context represents a section of a logical plan, with some available variables.
/// - A query is one context, with the query parameters as the available variables
/// - A context can contain an operator that in turn contains a new context (e.g. groupby)
pub struct Context {
    /// The ordering of operators in the context
    pub ordering: Vec<Key<Operator>>,
    /// The parameters for the context
    pub params: Vec<(Ident, Key<ScalarType>)>,
    /// if the context returns a value, then the return value operator
    /// INV is a [super::FlowOperator::Return]
    pub returnflow: Option<Key<Operator>>,
    pub discards: Vec<Key<Operator>>,
}

impl Context {
    pub fn from_params(params: impl Iterator<Item=(Ident, Key<ScalarType>)>) -> Self {
        Context {
            ordering: Vec::new(),
            params: params.collect(),
            returnflow: None,
            discards: Vec::new()
        }
    }

    pub fn set_return(&mut self, returnflow: Key<Operator>) {
        assert!(self.returnflow.is_none(), "Cannot set the return of a context twice");
        self.returnflow = Some(returnflow);
    }

    pub fn add_operator(&mut self, operator: Key<Operator>) {
        self.ordering.push(operator);
    }

    pub fn add_discard(&mut self, operator: Key<Operator>) {
        self.discards.push(operator);
    }
}

impl Plan {
    pub fn get_query(&self, key: Key<Query>) -> &Query {
        self.queries.get(key).unwrap()
    }

    pub fn get_context(&self, key: Key<Context>) -> &Context {
        self.contexts.get(key).unwrap()
    }

    pub fn get_mut_context(&mut self, key: Key<Context>) -> &mut Context {
        self.contexts.get_mut(key).unwrap()
    }
}