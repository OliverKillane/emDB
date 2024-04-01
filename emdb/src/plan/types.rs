//! Types for the logical plan.

use super::{GenArena, Key, LogicalPlan, Table, WithPlan};
use proc_macro2::Ident;
use quote::ToTokens;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use syn::Type;

#[derive(Clone)]
pub enum ConcRef<A: Clone> {
    Conc(A),

    /// A reference to another record/type
    /// - Used coalescing different records of the same type to point to the same concrete record
    /// INV: Not self-referential / no recursive types
    Ref(Key<ConcRef<A>>),
}

impl<A: Clone> ConcRef<A> {
    pub fn get_conc<'c, 'a: 'c, 'b: 'c>(&'b self, arena: &'a GenArena<Self>) -> &'c A {
        match self {
            ConcRef::Conc(c) => c,
            ConcRef::Ref(r) => arena.get(*r).unwrap().get_conc(arena),
        }
    }
}

pub type Record = ConcRef<RecordConc>;

#[derive(Clone)]
pub struct RecordConc {
    pub fields: HashMap<Ident, Key<ScalarType>>,
}

/// Used to describe types for [super::OperatorEdge] in the operator graph
#[derive(Clone)]
pub struct Data {
    pub fields: Key<Record>,
    pub stream: bool,
}

pub type ScalarType = ConcRef<ScalarTypeConc>;

#[derive(PartialEq, Eq, Clone)]
pub enum ScalarTypeConc {
    /// A reference to a row in a table, allows the user to interact wit row
    /// references while still allowing the backend to decide what they are.
    /// ```
    /// ref cool_table |> collect ~> return;
    /// ```
    /// - Can use different types of references depending table implementation
    /// chosen (e.g. key with generation, pointer, etc)
    TableRef(Key<Table>),

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

// TODO: parameterise the types so that we can reason about temporarily
// equal types -> i.e the concretes are parameterised by the record type

pub fn record_type_eq(lp: &LogicalPlan, r1: &Key<Record>, r2: &Key<Record>) -> bool {
    if r1 == r2 {
        return true;
    }

    let conc_r1 = lp.record_types.get(*r1).unwrap().get_conc(&lp.record_types);
    let conc_r2 = lp.record_types.get(*r2).unwrap().get_conc(&lp.record_types);

    let mut fields = conc_r1.fields.keys().collect::<HashSet<_>>();
    for (id, ty) in &conc_r2.fields {
        if let Some(r1_val) = conc_r1.fields.get(id) {
            if !scalar_type_eq(lp, r1_val, ty) {
                return false;
            }
        } else {
            return false;
        }
        fields.remove(id);
    }
    return fields.is_empty();
}

pub fn scalar_type_eq(lp: &LogicalPlan, t1: &Key<ScalarType>, t2: &Key<ScalarType>) -> bool {
    if t1 == t2 {
        return true;
    }

    let conc_t1 = lp.scalar_types.get(*t1).unwrap().get_conc(&lp.scalar_types);
    let conc_t2 = lp.scalar_types.get(*t1).unwrap().get_conc(&lp.scalar_types);

    match (conc_t1, conc_t2) {
        (ScalarTypeConc::TableRef(t1), ScalarTypeConc::TableRef(t2)) => t1 == t2,
        (
            ScalarTypeConc::Bag(r1) | ScalarTypeConc::Record(r1),
            ScalarTypeConc::Bag(r2) | ScalarTypeConc::Record(r2),
        ) => record_type_eq(lp, r1, r2),
        (ScalarTypeConc::Rust(rt1), ScalarTypeConc::Rust(rt2)) => rt1 == rt2,
        _ => false,
    }
}

pub fn append_field(
    lp: &mut LogicalPlan,
    existing: Key<Record>,
    new_field: Ident,
    new_value: Key<ScalarType>,
) -> Result<Key<Record>, Ident> {
    let mut existing_t = lp.get_record_type(existing).clone();

    if let Some((dup, _)) = existing_t.fields.remove_entry(&new_field) {
        Err(dup)
    } else {
        existing_t.fields.insert(new_field, new_value);
        Ok(lp.record_types.insert(ConcRef::Conc(existing_t)))
    }
}

// Helpers for type access
impl LogicalPlan {
    pub fn get_scalar_type(&self, k: Key<ScalarType>) -> &ScalarTypeConc {
        self.scalar_types
            .get(k)
            .unwrap()
            .get_conc(&self.scalar_types)
    }

    pub fn get_record_type(&self, k: Key<Record>) -> &RecordConc {
        self.record_types
            .get(k)
            .unwrap()
            .get_conc(&self.record_types)
    }
}

// boilerplate for displaying types ============================================

impl<'a, 'b> Display for WithPlan<'a, &'b Key<Record>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (field, ty) in self
            .plan
            .record_types
            .get(*self.extended)
            .unwrap()
            .get_conc(&self.plan.record_types)
            .fields
            .iter()
        {
            write!(
                f,
                "{field}: {}, ",
                WithPlan {
                    plan: self.plan,
                    extended: ty
                }
            )?;
        }
        write!(f, "}}")
    }
}

impl<'a, 'b> Display for WithPlan<'a, &'b Key<ScalarType>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let conc_t = self
            .plan
            .scalar_types
            .get(*self.extended)
            .unwrap()
            .get_conc(&self.plan.scalar_types);
        match conc_t {
            ScalarTypeConc::TableRef(t) => {
                write!(f, "ref {}", self.plan.tables.get(*t).unwrap().name)
            }
            ScalarTypeConc::Bag(b) => {
                write!(
                    f,
                    "collected {}",
                    WithPlan {
                        plan: self.plan,
                        extended: b
                    }
                )
            }
            ScalarTypeConc::Record(r) => WithPlan {
                plan: self.plan,
                extended: r,
            }
            .fmt(f),
            ScalarTypeConc::Rust(rt) => rt.to_token_stream().fmt(f),
        }
    }
}

impl<'a, 'b> Display for WithPlan<'a, &'b Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} of {}",
            if self.extended.stream {
                "stream"
            } else {
                "single record"
            },
            WithPlan {
                plan: self.plan,
                extended: &self.extended.fields
            }
        )
    }
}
