use std::{collections::HashSet, iter::once};

use quote::quote;
use quote_debug::Tokens;
use syn::{ExprCall, ExprTuple, Ident, ItemFn, ItemMod};

use crate::v2::columns::{Field, Group};

use super::{
    columns::{FieldName, Groups, PrimaryKind},
    namer::CodeNamer,
};

pub struct Predicate {
    pub alias: Ident,
    pub tokens: Tokens<ItemFn>,
}

impl Predicate {
    /// Generates the predicate function to be stored in the predicate module.
    pub fn generate_function<Primary: PrimaryKind>(
        &self,
        groups: &Groups<Primary>,
    ) -> Tokens<ItemFn> {
        let args = once(&groups.primary.fields)
            .chain(groups.assoc.iter().map(|grp| &grp.fields))
            .map(|fields| {
                fields
                    .imm_fields
                    .iter()
                    .chain(fields.mut_fields.iter())
                    .map(|Field { ty, name }| quote!(#name: &#ty))
            })
            .flatten();

        let name = &self.alias;
        let body = &self.tokens;

        quote! {
            fn #name(#(#args),*) -> bool {
                #body
            }
        }
        .into()
    }
}

/// Generate a module containing all predicates.
pub fn generate<Primary: PrimaryKind>(
    predicates: &[Predicate],
    groups: &Groups<Primary>,
    namer: &CodeNamer,
) -> Tokens<ItemMod> {
    let functions = predicates.iter().map(|pred| pred.generate_function(groups));
    let predicates_mod = namer.mod_predicates();

    quote! {
        pub mod #predicates_mod {
            #(#functions)*
        }
    }
    .into()
}

/// Generates a tuple of immutable borrows from a `.brw_mut(..)` method call,
/// but for fields in `new_fields` it uses the `update_value_name` struct instead.
/// - Allows for the row to be checked by predicates before it is committed to
///   the table entry.
pub fn generate_update_predicate_access<'a, Primary: PrimaryKind>(
    groups: &Groups<Primary>,
    new_fields: &HashSet<FieldName>,
    update_value_name: &Ident,
    namer: &CodeNamer,
) -> Tokens<ExprTuple> {
    let accesses = once((namer.name_primary_column(), &groups.primary.fields))
        .chain(
            groups
                .assoc
                .iter()
                .enumerate()
                .map(|(ind, grp)| (namer.name_assoc_column(ind), &grp.fields)),
        )
        .map(|(var_name, fields)| {
            fields
                .imm_fields
                .iter()
                .map(|f| (quote!(imm_data), f))
                .chain(fields.mut_fields.iter().map(|f| (quote!(mut_data), f)))
                .map(move |(access, Field { name, ty })| {
                    if new_fields.contains(&name) {
                        quote!(#update_value_name.#name)
                    } else {
                        quote!(&#var_name.#access.#name)
                    }
                })
        })
        .flatten();

    quote! {
        (#(#accesses),*)
    }
    .into()
}
