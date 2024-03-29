//! Types for the logical plan.

use std::{collections::HashMap, fmt::Display};
use super::{Key, Table, WithPlan};
use proc_macro2::Ident;
use quote::ToTokens;
use syn::Type;

/// Used to describe types for [super::OperatorEdge] in the operator graph
pub struct Data {
    pub fields: Key<Record>,
    pub stream: bool
}

pub enum Record {
    /// A record with named fields
    Concrete {
        fields: HashMap<Ident, ScalarType>,
    },

    /// A reference to another record
    /// - Used coalescing different records of the same type to point to the same concrete record
    /// INV: Not self-referential / no recursive types
    RecordRef(Key<Record>)
}

pub enum ScalarType {
    /// A reference to a row in a table, allows the user to interact wit row 
    /// references while still allowing the backend to decide what they are.
    /// ```
    /// ref cool_table |> collect ~> return; 
    /// ```
    /// - Can use different types of references depending table implementation 
    /// chosen (e.g. key with generation, pointer, etc)
    TableRef(Key<Table>),

    /// Used to reference common types, for example bags
    /// ```
    ///  |> collect(it as type foo)
    /// ```
    /// All usage of `type foo` will go to the same Key<ScalarType> which is a bag of the relevant type
    /// 
    /// Can be done for other synthetic types also.
    ///  
    /// INV: Not self-referential
    TypeRef(Key<ScalarType>), // TODO: necessity unclear (e.g. passing about a bag, could just copy bag type?)
    
    /// A collection of records in a container to be specified by the chosen 
    /// backend. Allows the plan to express the type, without specifying its 
    /// implementation.
    /// 
    /// ```
    ///  |> take(10) // with cardinality determination allocate bag for 10
    ///  |> collect(it as type my_fixed_bag)
    ///  ~> return;
    /// ```
    /// 
    /// ```
    /// use table_a
    ///  |> sort(it desc) // could reuse heap from sort as the bag type
    ///  |> collect(it as type my_variable_bag)
    ///  ~> return;
    /// ```
    Bag(Key<Record>),

    /// A record/struct of named fields
    Record(Key<Record>),

    /// A rust type propagated from the user
    /// - Can be from the user's code (e.g. a library)
    /// - Can be incorrect (need to propagate spans to backend for rustc to 
    ///   report)
    Rust(Type),
}

impl<'a, 'b> Display for WithPlan<'a, &'b Record> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.extended {
            Record::Concrete { fields } => {
                write!(f, "{{")?;
                for (a, b) in fields {
                    write!(
                        f,
                        "{a}: {}, ",
                        WithPlan {
                            plan: self.plan,
                            extended: b
                        }
                    )?;
                }
                write!(f, "}}")
            },
            Record::RecordRef(r) => WithPlan {
                plan: self.plan,
                extended: self.plan.record_types.get(*r).unwrap(),
            }.fmt(f)
        }
    }
}

impl<'a, 'b> Display for WithPlan<'a, &'b ScalarType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.extended {
            ScalarType::TableRef(t) => {
                // INV: the type is valid => the index is in the plan
                write!(f, "ref {}", self.plan.tables.get(*t).unwrap().name)
            }
            ScalarType::TypeRef(t) => {
                // INV: the type is valid => the index in the types
                write!(f, "type {}", WithPlan { plan: self.plan, extended: self.plan.scalar_types.get(*t).unwrap() })
            }
            ScalarType::Bag(b) => {
                write!(f, "collected {{{}}}", WithPlan {
                    plan: self.plan,
                    extended: self.plan.record_types.get(*b).unwrap()
                })
            }
            ScalarType::Record(r) => {
                WithPlan {
                    plan: self.plan,
                    extended: self.plan.record_types.get(*r).unwrap(),
                }
                .fmt(f)
            }
            ScalarType::Rust(t) => t.to_token_stream().fmt(f),
        }
    }
}

impl<'a, 'b> Display for WithPlan<'a, &'b Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} of {}", if self.extended.stream {"stream"} else {"single record"}, WithPlan {
            plan: self.plan,
            extended: self.plan.record_types.get(self.extended.fields).unwrap()
        })
    }
}