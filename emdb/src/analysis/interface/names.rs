use crate::plan;
use proc_macro2::{Ident, Span};

/// A stateless namingname generator
pub trait ItemNamer {
    fn record_type(key: plan::Key<plan::RecordType>) -> Ident;
    fn scalar_type(key: plan::Key<plan::ScalarType>) -> Ident;
    fn table(key: plan::Key<plan::Table>) -> Ident;
    fn record_field(rf: &plan::RecordField) -> Ident;
    
    fn context(key: plan::Key<plan::Context>) -> Ident;
    fn context_pattern(key: plan::Key<plan::Context>) -> Ident;

    fn operator(key: plan::Key<plan::Operator>) -> Ident;
    fn operator_pattern(key: plan::Key<plan::Operator>) -> Ident;
}

pub struct DefaultNamer;

fn name<A>(id: plan::Key<A>, prefix: &str) -> Ident {
    Ident::new(&format!("{}{}", prefix, id.to_idx()), Span::call_site())
}

impl ItemNamer for DefaultNamer {
    fn record_type(key: plan::Key<plan::RecordType>) -> Ident {
        name(key, "RecordType") 
    }

    fn scalar_type(key: plan::Key<plan::ScalarType>) -> Ident {
        name(key, "ScalarType")
    }

    fn table(key: plan::Key<plan::Table>) -> Ident {
        name(key, "Table")
    }

    fn record_field(rf: &plan::RecordField) -> Ident {
        match rf {
            plan::RecordField::User(i) => i.clone(),
            plan::RecordField::Internal(i) => Ident::new(&format!("recordfield_internal_id_{i}"), Span::call_site()),
        }
    }

    fn context(key: plan::Key<plan::Context>) -> Ident {
        name(key, "Context")
    }
    fn context_pattern(key: plan::Key<plan::Context>) -> Ident {
        name(key, "ContextPattern")
    }

    fn operator(key: plan::Key<plan::Operator>) -> Ident {
        name(key, "Operator")
    }
    fn operator_pattern(key: plan::Key<plan::Operator>) -> Ident {
        name(key, "OperatorPattern")
    }
}