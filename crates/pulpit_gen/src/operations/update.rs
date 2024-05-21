use proc_macro2::Span;
use quote::quote;
use quote_debug::Tokens;
use std::collections::{HashMap, HashSet};
use syn::{
    ExprAssign, ExprMethodCall, Ident, ImplItemFn, ItemEnum, ItemImpl, ItemMod, ItemStruct,
    ItemTrait, TraitItemFn, Variant,
};

use crate::{
    columns::{FieldIndex, FieldName, Groups, PrimaryKind},
    namer::CodeNamer,
    predicates::{generate_update_predicate_access, Predicate},
    uniques::Unique,
};

use super::SingleOp;

/// An update operation, replacing [`Update::fields`] with new values.
/// - Named for the user by [`Update::alias`]
pub struct Update {
    pub fields: HashSet<Ident>,
    pub alias: Ident,
}

pub fn generate<Primary: PrimaryKind>(
    updates: &[Update],
    groups: &Groups<Primary>,
    uniques: &HashMap<FieldName, Unique>,
    predicates: &[Predicate],
    namer: &CodeNamer,
) -> SingleOp {
    let modules = updates
        .iter()
        .map(|update| update.generate_mod(groups, uniques, predicates, namer));
    let trait_fns = updates.iter().map(|update| update.generate_trait_fn(namer));
    let impl_fns = updates
        .iter()
        .map(|update| update.generate_trait_impl_fn(namer, groups, uniques, predicates));

    let trait_name = namer.trait_update();
    let update_mod = namer.mod_update();
    let window_struct = namer.struct_window();

    SingleOp {
        op_mod: quote! {
            pub mod #update_mod {
                #(#modules)*
            }
        }
        .into(),
        op_trait: quote! {
            pub trait #trait_name {
                #(#trait_fns)*
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> #trait_name for #window_struct<'imm> {
                #(#impl_fns)*
            }
        }
        .into(),
    }
}

impl Update {
    fn generate_mod<Primary: PrimaryKind>(
        &self,
        groups: &Groups<Primary>,
        uniques: &HashMap<FieldName, Unique>,
        predicates: &[Predicate],
        namer: &CodeNamer,
    ) -> Tokens<ItemMod> {
        fn generate_unique_error_variants<'a>(
            unique_indexes: impl Iterator<Item = (&'a Ident, &'a Unique)>,
            namer: &CodeNamer,
        ) -> Vec<Tokens<Variant>> {
            let pulpit_path = namer.pulpit_path();
            unique_indexes
                .map(|(_, unique)| {
                    let variant = &unique.alias;
                    quote!(
                        #variant(#pulpit_path::access::UniqueConflict)
                    )
                    .into()
                })
                .collect()
        }

        fn generate_predicate_error_variants(predicates: &[Predicate]) -> Vec<Tokens<Variant>> {
            predicates
                .iter()
                .map(|pred| {
                    let variant = &pred.alias;
                    quote!(
                        #variant
                    )
                    .into()
                })
                .collect()
        }

        // names for code generation
        let pulpit_path = namer.pulpit_path();
        let update_error = namer.mod_update_enum_error();
        let key_error = namer.type_key_error();
        let update_struct_name = namer.mod_update_struct_update();
        let update_name = &self.alias;

        // get the unique error types
        let unique_indexes = uniques
            .iter()
            .filter(|(field, _)| self.fields.contains(field));

        let unique_errors = generate_unique_error_variants(unique_indexes, namer);
        let predicate_errors = generate_predicate_error_variants(predicates);

        let struct_fields = self.fields.iter().map(|f| {
            let ty = groups.get_type(groups.get_field_index(f).unwrap()).unwrap();
            quote!(#f : #ty)
        });

        quote! {
            pub mod #update_name {
                pub enum #update_error {
                    #key_error,
                    #(#unique_errors),*
                    #(#predicate_errors),*
                }

                pub struct #update_struct_name {
                    #(pub #struct_fields),*
                }
            }
        }
        .into()
    }

    fn generate_trait_fn(&self, namer: &CodeNamer) -> Tokens<TraitItemFn> {
        let update_struct_name = namer.mod_update_struct_update();
        let update_error = namer.mod_update_enum_error();
        let update_name = &self.alias;

        quote! {
            fn #update_name(&mut self, update: #update_struct_name) -> Result<(), #update_error>;
        }
        .into()
    }

    fn generate_trait_impl_fn<Primary: PrimaryKind>(
        &self,
        namer: &CodeNamer,
        groups: &Groups<Primary>,
        uniques: &HashMap<FieldName, Unique>,
        predicates: &[Predicate],
    ) -> Tokens<ImplItemFn> {
        let update_struct_name = namer.mod_update_struct_update();
        let update_error = namer.mod_update_enum_error();
        let update_name = &self.alias;
        let primary = namer.name_primary_column();
        let columns = namer.table_member_columns();
        let key_type = namer.type_key();
        let predicate_mod = namer.mod_predicates();
        let key_error = namer.type_key_error();
        let update_var = Ident::new("update", Span::call_site());

        // Generate the table access to primary, and all associated!
        let assoc_brw_muts = (0..groups.assoc.len()).map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(let #name = self.#columns.#name.brw_mut(index))
        });
        let table_access = quote! {
            let Entry { index, data: #primary } = match self.#columns.#primary.brw_mut(key) {
                Ok(entry) => entry,
                Err(e) => return Err(#update_error::#key_error),
            }
            #(#assoc_brw_muts;)*
        };

        // Pass borrow of all fields to the predicate (check if it will be valid)
        // needs to include new updated values
        let predicate_args =
            generate_update_predicate_access(groups, &self.fields, &update_var, namer);
        let predicate_checks = predicates.iter().map(|pred| {
            let pred = &pred.alias;
            quote! {
                if !#predicate_mod::#pred #predicate_args {
                    return Err(#update_error::#pred);
                }
            }
        });

        let uniques_member = namer.table_member_uniques();
        let mut undo_prev_fields: Vec<Tokens<ExprMethodCall>> = Vec::new();
        let mut unique_updates: Vec<Tokens<ExprAssign>> = Vec::new();
        for (field, Unique { alias }) in uniques.iter() {
            let field_index = groups.idents.get(field).unwrap();
            let from_data = match field_index {
                FieldIndex::Primary(_) => namer.name_primary_column(),
                FieldIndex::Assoc { assoc_ind, .. } => namer.name_assoc_column(*assoc_ind),
            };

            let mutability = if field_index.is_imm() {
                quote!(imm_data)
            } else {
                quote!(mut_data)
            };

            unique_updates.push(quote!{
                let #alias = match self.#uniques_member.#alias.replace(&update.#field, &#from_data.#mutability.#field, key) {
                    Ok(old_val) => old_val,
                    Err(_) => {
                        #(#undo_prev_fields;)*
                        return Err(#update_error::#alias)
                    },
                }
            }.into());

            undo_prev_fields.push(
                quote! {
                    self.#uniques_member.#alias.undo_replace(#alias, &update.#field, key)
                }
                .into(),
            )
        }

        let update_pairs = self.fields.iter().map(|field| {
            let field_index = groups.idents.get(field).unwrap();
            let name_id = match field_index {
                FieldIndex::Primary(_) => namer.name_primary_column(),
                FieldIndex::Assoc { assoc_ind, .. } => namer.name_assoc_column(*assoc_ind),
            };

            (field, quote!(#name_id.mut_data.#field))
        });

        let commit_updates = if Primary::TRANSACTIONS {
            let updates = update_pairs.map(|(field, mut_access)| {
                quote! {
                    std::mem::swap(&mut #mut_access, &mut update.#field);
                }
            });

            let log_type = namer.mod_transactions_enum_log();
            let trans_mod = namer.mod_transactions();
            let transactions_member = namer.table_member_transactions();
            let transaction_update_type = namer.mod_transactions_enum_update();
            quote! {
                let mut update = update;
                #(#updates;)*

                if self.#transactions_member.append {
                    self.#transactions_member.log.push(#trans_mod::#log_type::Update(#trans_mod::#transaction_update_type::#update_name(update)));
                }
            }
        } else {
            let updates = update_pairs.map(|(field, mut_access)| {
                quote! {
                    *#mut_access = update.#field
                }
            });
            quote! {
                #(#updates;)*
            }
        };

        quote! {
            fn #update_name(&mut self, #update_var: #update_struct_name, key: #key_type) -> Result<(), #update_error> {
                #table_access
                #(#predicate_checks)*
                #(#unique_updates)*
                #commit_updates
                Ok(())
            }
        }
        .into()
    }
}
