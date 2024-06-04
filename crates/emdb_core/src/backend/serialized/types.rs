//! # Generate types for the simple backend
//!
//! ## Type Aliases vs resolving.
//! When translating types from the [`plan::Key`]s to [`plan::RecordType`] and
//! [`plan::ScalarType`], we have two options for generating a name.
//!
//! 1. Generate type aliases from the key and use them, generate a type aliases
//!    for every single [`plan::ScalarType`] and [`plan::RecordType`] with type
//!    aliases for [`plan::ConcRef::Ref`].
//! 2. Traverse the graph each time to get the actual type (or some more efficient
//!    collapsing of references to [`plan::ConcRef::Conc`] types).
//!
//! While (1.) is more efficient for the macro, it leaves more work for the rust
//! compiler, and generates large amounts of type aliases, which make human
//! reading & debugging of the generated code more difficult.
//!
//! On balance I decided to go with (2.), though it complicates some code (need
//! to access the plan, and the [`pulpit::gen::operations::get`] types to gets
//! the tokens for a type)

use quote::{quote, ToTokens};
use quote_debug::Tokens;
use std::collections::{HashMap, HashSet};
use syn::{Ident, ItemStruct, Type};

use super::namer::SimpleNamer;
use crate::plan;

/// Gets all the record types that need to be declared public.
/// - Scalar types do not need this, they are either type aliases (no need to
///   be public, they are exposed to the user as the type they alias)
/// - References to types are aliases, so are also not included.
fn public_record_types(lp: &plan::Plan) -> HashSet<plan::ImmKey<'_, plan::RecordType>> {
    fn recursive_collect_record<'imm>(
        lp: &'imm plan::Plan,
        attrs: &mut HashSet<plan::ImmKey<'imm, plan::RecordType>>,
        mut key: plan::Key<plan::RecordType>,
    ) {
        loop {
            match lp.get_record_type(key) {
                plan::ConcRef::Ref(k) => key = *k,
                plan::ConcRef::Conc(record) => {
                    attrs.insert(plan::ImmKey::new(key, lp));
                    for ty_key in record.fields.values() {
                        recursive_collect_scalar(lp, attrs, *ty_key)
                    }
                    return;
                }
            }
        }
    }

    fn recursive_collect_scalar<'imm>(
        lp: &'imm plan::Plan,
        attrs: &mut HashSet<plan::ImmKey<'imm, plan::RecordType>>,
        mut key: plan::Key<plan::ScalarType>,
    ) {
        loop {
            match lp.get_scalar_type(key) {
                plan::ConcRef::Ref(k) => key = *k,
                plan::ConcRef::Conc(c) => {
                    // Scalar types are always either aliases, or using types already available to the user, so no need to make them public.
                    match c {
                        plan::ScalarTypeConc::Bag(r) | plan::ScalarTypeConc::Record(r) => {
                            recursive_collect_record(lp, attrs, *r)
                        }
                        plan::ScalarTypeConc::TableRef(_)
                        | plan::ScalarTypeConc::TableGet { .. } => (
                            // These are already specified to be public, so no need to additionally make public here
                        ),
                        plan::ScalarTypeConc::Rust { .. } => (
                            // The user provided types are already available to the user, no need to make public here
                        ),
                    }
                    return;
                }
            }
        }
    }

    let mut public_records = HashSet::new();
    for (_, query) in &lp.queries {
        if let Some(ret_type) = lp.get_context(query.ctx).get_return_type(lp) {
            recursive_collect_record(lp, &mut public_records, ret_type);
        }
    }

    public_records
}

/// Generates the tokens for a given scalar type.
/// - Needs to consider the values transformed by [`pulpit::gen::operations::get`]
///   which are determined after the table structure is chosen.
/// - Generates types with lifetimes ([`SimpleNamer::db_lifetime`] and
///   [`SimpleNamer::qy_lifetime`]) usable only in a [`plan::TypeContext::Query`]
///   context.
pub fn generate_scalar_type<'imm>(
    lp: &'imm plan::Plan,
    get_types: &HashMap<plan::Idx<'imm, plan::Table>, HashMap<Ident, Tokens<Type>>>,
    key: plan::Key<plan::ScalarType>,
    namer: &SimpleNamer,
) -> Tokens<Type> {
    match lp.get_scalar_type_conc(key) {
        plan::ScalarTypeConc::TableRef(tk) => {
            let table_name = &lp.get_table(*tk).name;
            let SimpleNamer {
                pulpit: pulpit::gen::namer::CodeNamer { type_key, .. },
                mod_tables,
                ..
            } = namer;
            quote! {
                #mod_tables::#table_name::#type_key
            }
            .into()
        }
        plan::ScalarTypeConc::TableGet { table, field } => get_types
            .get(&plan::Idx::new(*table, lp))
            .unwrap()
            .get(field.get_field())
            .unwrap()
            .clone(),
        plan::ScalarTypeConc::Bag(r) => {
            let rec_name = namer.record_name_lifetimes(*lp.get_record_conc_index(*r));
            quote!(Vec<#rec_name>).into()
        }
        plan::ScalarTypeConc::Record(r) => {
            namer.record_name_lifetimes(*lp.get_record_conc_index(*r))
        }
        plan::ScalarTypeConc::Rust {
            type_context: _, // can be used on either datastore or query types, wraps in the lifetimes required for query
            ty,
        } => ty.to_token_stream().into(),
    }
}

/// Gets the name of a record type to allow for its construction
/// - Does not include lifetime parameters, just the struct name.
pub fn generate_record_name(
    lp: &plan::Plan,
    key: plan::Key<plan::RecordType>,
    namer: &SimpleNamer,
) -> Tokens<Type> {
    let index = lp.get_record_conc_index(key);
    namer.record_name(*index)
}

/// Generates the definitions for record types
/// - structs used to represent [`plan::RecordConc`]
/// - publicity is determined traversing the return type of queries
///
/// Each record type needs a [`SimpleNamer::phantom_field`] to ensure the query lifetime
/// parameters are bound (we do not avalyse types provided by the user to check for usage).
pub fn generate_record_definitions<'imm>(
    lp: &'imm plan::Plan,
    get_types: &'imm HashMap<plan::Idx<'imm, plan::Table>, HashMap<Ident, Tokens<Type>>>,
    namer: &'imm SimpleNamer,
) -> impl Iterator<Item = Tokens<ItemStruct>> + 'imm {
    let public_records = public_record_types(lp);

    let SimpleNamer {
        qy_lifetime,
        db_lifetime,
        phantom_field,
        ..
    } = namer;

    lp.record_types.iter().filter_map(move |(key, rec)| {
        match rec {
            plan::ConcRef::Conc(rec) => {
                let name = namer.record_name(key);
                let pub_tks = if public_records.contains(&plan::ImmKey::new(key, lp)) {
                    quote!(pub )
                } else {
                    quote!()
                };
                let members = rec.fields.iter().map(
                    |(field, ty)| {
                        let fieldname = namer.transform_field_name(field);
                        let ty_tks = generate_scalar_type(lp, get_types, *ty, namer);
                        quote!{
                            #pub_tks #fieldname: #ty_tks
                        }
                    }
                );

                Some(quote!{
                    #[derive(Clone)]
                    #pub_tks struct #name<#db_lifetime, #qy_lifetime> {
                        #(#members,)*
                        #phantom_field: std::marker::PhantomData<(&#db_lifetime (), &#qy_lifetime ())>,
                    }
                }.into())
            },
            plan::ConcRef::Ref(_) => None,
        }
    })
}
