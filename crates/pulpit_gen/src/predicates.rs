use std::iter::once;

use quote::quote;
use quote_debug::Tokens;
use syn::{Expr, ExprStruct, Ident, ItemFn, ItemMod};

use crate::groups::Field;

use super::{
    groups::{FieldName, Groups},
    namer::CodeNamer,
};

pub struct Predicate {
    pub alias: Ident,
    pub tokens: Tokens<Expr>,
}

impl Predicate {
    /// Generates the predicate function to be stored in the predicate module.
    pub fn generate_function(&self, groups: &Groups, namer: &CodeNamer) -> Tokens<ItemFn> {
        let CodeNamer {
            mod_borrow,
            mod_borrow_struct_borrow,
            ..
        } = namer;
        let struct_args = if groups.idents.is_empty() {
            quote!(..)
        } else {
            let args = groups.idents.keys();
            quote!(#(#args),*)
        };

        let name = &self.alias;
        let body = &self.tokens;
        quote! {
            #[inline(always)]
            pub fn #name(super::#mod_borrow::#mod_borrow_struct_borrow { #struct_args }: super::#mod_borrow::#mod_borrow_struct_borrow) -> bool {
                #body
            }
        }
        .into()
    }
}

/// Generate a module containing all predicates.
pub fn generate(predicates: &[Predicate], groups: &Groups, namer: &CodeNamer) -> Tokens<ItemMod> {
    let functions = predicates
        .iter()
        .map(|pred| pred.generate_function(groups, namer));
    let mod_predicates = &namer.mod_predicates;

    quote! {
        mod #mod_predicates {
            #(#functions)*
        }
    }
    .into()
}

/// Generates a tuple of immutable borrows from a `.brw_mut(..)` method call,
/// but for fields in `new_fields` it uses the `update_value_name` struct instead.
/// - Allows for the row to be checked by predicates before it is committed to
///   the table entry.
pub fn generate_update_predicate_access(
    groups: &Groups,
    new_fields: &[FieldName],
    update_value_name: &Ident,
    namer: &CodeNamer,
) -> Tokens<ExprStruct> {
    let CodeNamer {
        mod_borrow,
        mod_borrow_struct_borrow,
        name_phantom_member,
        ..
    } = namer;

    let accesses = once((namer.name_primary_column.clone(), &groups.primary.fields))
        .chain(
            groups
                .assoc
                .iter()
                .enumerate()
                .map(|(ind, grp)| (namer.name_assoc_column(ind), &grp.fields)),
        )
        .flat_map(|(var_name, fields)| {
            fields
                .imm_fields
                .iter()
                .map(|f| (quote!(imm_data), f))
                .chain(fields.mut_fields.iter().map(|f| (quote!(mut_data), f)))
                .map(move |(access, Field { name, ty: _ })| {
                    if new_fields.contains(name) {
                        quote!(#name: &#update_value_name.#name)
                    } else {
                        quote!(#name: &#var_name.#access.#name)
                    }
                })
        })
        .collect::<Vec<_>>();

    let access_fields = if accesses.is_empty() {
        quote!(#name_phantom_member: std::marker::PhantomData)
    } else {
        quote!(#(#accesses),*)
    };

    quote! {
        #mod_borrow::#mod_borrow_struct_borrow {
            #access_fields
        }
    }
    .into()
}
