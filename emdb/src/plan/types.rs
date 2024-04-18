//! #Types for the logical plan.
//!
//! ## Type System
//! ### Supported Types
//! Records are supported as the main data type wrapper (expressions have the
//! context of the record type they are in)
//!
//! Types supported include rust types, ref types (backend decides implementation)
//! and wrappers such as [`ScalarTypeConc::Record`], and [`ScalarTypeConc::Bag`]
//!
//! It is important to allow all rust types to be supported, to allow frontends
//! maximum flexibility in the types they use.
//!
//! ### Equality & [Coercion](coerce_record_type)
//! Types are defined through a graph of concrete types, and references to other
//! types (using [`ConcRef`]).
//!
//! The logical plan, and consuming backends need to be able to reason about
//! two properties of types:
//!
//! 1. Logical Type Equality (used for semantic analysis) asks: is there an
//!    implementation of these types for which they are equal?
//!    - two different bags of an equal type are equal (use the same bag data structure)
//!    - two references to the same table are equal (use the same reference type)
//!    - two records with the same fields are equal (use the same wrapping record data structure)
//!    This is the type of equality we use for semantic analysis.   
//!
//! 2. Implementation Type Equality (used for code generation) asks: do the
//!    implementations of these types *have* to be the same?
//!    - two bags of the same type, that are in streams never unioned, can be
//!      implemented with different wrapper (e.g. one is a fixed size bag, the
//!      other a vector)
//!    - two records of the same type could be separately optimised
//!
//! Implementation Equality implies Type equality (but not the reverse).
//!
//! Some special rules are also considered:
//! - [`ScalarTypeConc::TableRef`] is actually just another kind of Ref (as in
//!   [`ConcRef`]), but to a table. Backends need to choose the same type for
//!   all references to the same table as users can return and send refs as
//!   parameters.
//! - backends implementations of [`ScalarTypeConc::Rust`] need to ensure they
//!   comply with expressions written using these types (e.g. wrap, but unwrap
//!   to let users access inside expressions).  
//!
//! We use the [`coerce_record_type`] and [`coerce_scalar_type`] functions to
//! coerce logically equal types for operators such as union.
//!
//! ## Debugging
//! Using the [`Planviz`](crate::backend) backend with types displayed
//! shows the type graph.
//! - Unused references are legal (cause no harm, no point in garbage collecting),
//!   caused by coersion where an inner scalar type is coerced, but is not actually
//!   used anywhere, or when a type is used by an operator that is removed.
//! - Cycles are *very bad*, avoid at all cost. Can cause stackoverflow on
//!   compilation, [`ConcRef`] are assumed to be non-cyclic.

use super::{GenArena, Key, Plan, Table, With};
use proc_macro2::Ident;
use quote::ToTokens;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use syn::Type;

#[enumtrait::quick_from]
#[derive(Clone)]
pub enum ConcRef<A: Clone> {
    Conc(A),

    /// A reference to another record/type
    /// - Used coalescing different records of the same type to point to the same concrete record
    /// INV: Not self-referential / no recursive types / no cycles
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

fn get_conc_index<A: Clone>(
    arena: &GenArena<ConcRef<A>>,
    mut key: Key<ConcRef<A>>,
) -> Key<ConcRef<A>> {
    while let ConcRef::Ref(r) = arena.get(key).unwrap() {
        key = *r;
    }
    key
}

/// Separates user specified identifiers (which will have spans from the
/// original code positions) from internally generated identifiers.
#[enumtrait::quick_from]
#[derive(Clone, Hash, PartialEq, Eq)]
pub enum RecordField {
    User(Ident),
    Internal(usize),
}

pub type RecordType = ConcRef<RecordConc>;

#[derive(Clone)]
pub struct RecordConc {
    pub fields: HashMap<RecordField, Key<ScalarType>>,
}

/// Used to describe types for [`super::DataFlow`] in the operator graph
#[derive(Clone)]
pub struct Data {
    pub fields: Key<RecordType>,
    pub stream: bool,
}

pub type ScalarType = ConcRef<ScalarTypeConc>;

#[derive(PartialEq, Eq, Clone)]
pub enum ScalarTypeConc {
    /// A reference to a row in a table, allows the user to interact wit row
    /// references while still allowing the backend to decide what they are.
    /// ```
    /// # use emdb::emql;
    /// # emql! {
    /// #     table foos { x: i32 }
    /// #     query foo_query() {
    /// ref foos as foo_ref
    ///     |> collect(it as type foo_row)
    ///     ~> return;
    /// #     }
    /// # }
    /// ```
    /// - Can use different types of references depending table implementation
    /// chosen (e.g. key with generation, pointer, etc)
    TableRef(Key<Table>),

    /// A collection of records in a container to be specified by the chosen
    /// backend. Allows the plan to express the type, without specifying its
    /// implementation.
    ///
    /// ```
    /// # use emdb::emql;
    /// # emql! {
    /// #     table foos { x: i32 }
    /// #     query foo_query() {
    /// #     use foos
    /// |> take(10) // with cardinality determination allocate bag for 10
    /// |> collect(it as type my_fixed_bag)
    /// #         ~> return;
    /// #     }
    /// # }
    /// ```
    ///
    /// ```
    /// # use emdb::emql;
    /// # emql! {
    /// #     table foos { x: i32 }
    /// #     query foo_query() {
    /// #         use foos
    /// |> sort(x desc) // could reuse heap from sort as the bag type
    /// |> collect(it as type my_variable_bag)
    /// #             ~> return;
    /// #     }
    /// # }
    /// ```
    Bag(Key<RecordType>),

    /// A record/struct of named fields
    Record(Key<RecordType>),

    /// A rust type propagated from the user
    /// - Can be from the user's code (e.g. a library)
    /// - Can be incorrect (need to propagate spans to backend for rustc to
    ///   report)
    Rust(Type),
}

/// Check two record types are equal.
/// - The indicies in the types arenas (in [`Plan`]) may be different, if you want to coerse, use [`coerce_record_type`]
pub fn record_type_eq(lp: &Plan, r1: &Key<RecordType>, r2: &Key<RecordType>) -> bool {
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
    fields.is_empty()
}

/// Check two scalar types are equal.
/// - The indicies in the types arenas (in [`Plan`]) may be different, if you want to coerse, use [`coerce_scalar_type`]
pub fn scalar_type_eq(lp: &Plan, t1: &Key<ScalarType>, t2: &Key<ScalarType>) -> bool {
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

/// Coerce two scalar types for equality -> same index
pub fn coerce_scalar_type(lp: &mut Plan, conform_to: Key<ScalarType>, change: Key<ScalarType>) {
    let conform_index = get_conc_index(&lp.scalar_types, conform_to);
    let conforming_index = get_conc_index(&lp.scalar_types, change);

    if conform_index != conforming_index {
        let conform_scalar = lp.get_scalar_type(conform_index);
        let conforming_scalar = lp.get_scalar_type(conforming_index);

        if let (ScalarTypeConc::Bag(r1), ScalarTypeConc::Bag(r2))
        | (ScalarTypeConc::Record(r1), ScalarTypeConc::Record(r2)) =
            (conform_scalar, conforming_scalar)
        {
            coerce_record_type(lp, *r1, *r2);
        }
        *lp.scalar_types.get_mut(conforming_index).unwrap() = ConcRef::Ref(conform_index);
    }
}

/// Coerce two scalar record for equality -> same index
///
/// We do this to merge dataflows of the same type, and have those types be [`Key`] equal.
/// - Thus a backend can decide how to implement a bag, or record, and all references to the same types are updated.
/// - Other types in the plan may be equal, but if they are not the same index, backends can choose different concrete implementations.
/// - This is more efficient than just constantly re-checking names for types, and exposing said names to the backend.
///
/// be very careful however:
/// - cycles are bad, to debug cycles, disable printing of operator nodes in the [`Planviz`](`crate::backend`) backend,
///   and check for cycles in the grpah.
/// - cycles can affect debug printing of [`Data`] types, which can cause segfaults to occur on stack overflow.
pub fn coerce_record_type(lp: &mut Plan, conform_to: Key<RecordType>, change: Key<RecordType>) {
    let conform_index = get_conc_index(&lp.record_types, conform_to);
    let conforming_index = get_conc_index(&lp.record_types, change);

    if conform_index != conforming_index {
        let conform_record = lp.get_record_type(conform_index);
        let conforming_record = lp.get_record_type(conforming_index);

        let conform_fields = conform_record
            .fields
            .iter()
            .map(|(field, ty)| (*ty, *conforming_record.fields.get(field).unwrap()))
            .collect::<Vec<_>>();
        for (to, from) in &conform_fields {
            coerce_scalar_type(lp, *to, *from);
        }
        *lp.record_types.get_mut(conforming_index).unwrap() = ConcRef::Ref(conform_index);
    }
}

impl Plan {
    pub fn get_scalar_type(&self, k: Key<ScalarType>) -> &ScalarTypeConc {
        self.scalar_types
            .get(k)
            .unwrap()
            .get_conc(&self.scalar_types)
    }

    pub fn get_record_type(&self, k: Key<RecordType>) -> &RecordConc {
        self.record_types
            .get(k)
            .unwrap()
            .get_conc(&self.record_types)
    }
}

impl Display for RecordField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordField::User(id) => id.fmt(f),
            RecordField::Internal(id) => write!(f, "INTERNAL_ID<{}>", id),
        }
    }
}

impl<'a, 'b> Display for With<'a, &'b Key<RecordType>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (field, ty) in &self
            .plan
            .record_types
            .get(*self.extended)
            .unwrap()
            .get_conc(&self.plan.record_types)
            .fields
        {
            write!(
                f,
                "{field}: {}, ",
                With {
                    plan: self.plan,
                    extended: ty
                }
            )?;
        }
        write!(f, "}}")
    }
}

impl<'a, 'b> Display for With<'a, &'b Key<ScalarType>> {
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
                    With {
                        plan: self.plan,
                        extended: b
                    }
                )
            }
            ScalarTypeConc::Record(r) => With {
                plan: self.plan,
                extended: r,
            }
            .fmt(f),
            ScalarTypeConc::Rust(rt) => rt.to_token_stream().fmt(f),
        }
    }
}

impl<'a, 'b> Display for With<'a, &'b Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            if self.extended.stream { "stream" } else { "" },
            With {
                plan: self.plan,
                extended: &self.extended.fields
            }
        )
    }
}
