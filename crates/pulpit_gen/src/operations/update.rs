use proc_macro2::Span;
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprLet, ExprMethodCall, Ident, ImplItemFn, ItemMod, Variant};

use crate::{
    groups::{FieldIndex, Groups},
    namer::CodeNamer,
    predicates::{generate_update_predicate_access, Predicate},
    uniques::Unique,
};

use super::SingleOp;

/// An update operation, replacing [`Update::fields`] with new values.
/// - Named for the user by [`Update::alias`]
pub struct Update {
    pub fields: Vec<Ident>,
    pub alias: Ident,
}

pub fn generate(
    updates: &[Update],
    groups: &Groups,
    uniques: &[Unique],
    predicates: &[Predicate],
    namer: &CodeNamer,
    transactions: bool,
) -> SingleOp {
    let CodeNamer {
        mod_update,
        struct_window,
        ..
    } = namer;

    let modules = updates
        .iter()
        .map(|update| update.generate_mod(groups, uniques, predicates, namer));
    let impl_fns = updates.iter().map(|update| {
        update.generate_trait_impl_fn(namer, groups, uniques, predicates, transactions)
    });

    SingleOp {
        op_mod: quote! {
            pub mod #mod_update {
                #(#modules)*
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> #struct_window<'imm> {
                #(#impl_fns)*
            }
        }
        .into(),
    }
}

impl Update {
    fn generate_mod(
        &self,
        groups: &Groups,
        uniques: &[Unique],
        predicates: &[Predicate],
        namer: &CodeNamer,
    ) -> Tokens<ItemMod> {
        fn generate_unique_error_variants<'a>(
            unique_indexes: impl Iterator<Item = &'a Unique>,
        ) -> Vec<Tokens<Variant>> {
            unique_indexes
                .map(|unique| {
                    let variant = &unique.alias;
                    quote!(
                        #variant
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
        let CodeNamer {
            mod_update_enum_error,
            type_key_error,
            mod_update_struct_update,
            ..
        } = namer;

        let update_name = &self.alias;

        // get the unique error types
        let unique_indexes = uniques
            .iter()
            .filter(|uniq| self.fields.contains(&uniq.field));
        let unique_errors = generate_unique_error_variants(unique_indexes);
        let predicate_errors = generate_predicate_error_variants(predicates);

        let struct_fields = self.fields.iter().map(|f| {
            let ty = groups.get_type(groups.get_field_index(f).unwrap()).unwrap();
            quote!(#f : #ty)
        });

        let extra_comma = if unique_errors.is_empty() || predicate_errors.is_empty() {
            quote!()
        } else {
            quote!(,)
        };

        quote! {
            pub mod #update_name {
                #[derive(Debug)]
                pub enum #mod_update_enum_error {
                    #type_key_error,
                    #(#unique_errors),* #extra_comma
                    #(#predicate_errors),*
                }

                pub struct #mod_update_struct_update {
                    #(pub #struct_fields),*
                }
            }
        }
        .into()
    }

    fn generate_trait_impl_fn(
        &self,
        namer: &CodeNamer,
        groups: &Groups,
        uniques: &[Unique],
        predicates: &[Predicate],
        transactions: bool,
    ) -> Tokens<ImplItemFn> {
        let CodeNamer {
            mod_update,
            mod_update_struct_update,
            mod_update_enum_error,
            name_primary_column,
            struct_table_member_columns: table_member_columns,
            type_key,
            mod_predicates,
            type_key_error,
            pulpit_path,
            mod_transactions_enum_logitem,
            mod_transactions,
            mod_transactions_enum_logitem_variant_update,
            struct_table_member_transactions: table_member_transactions,
            mod_transactions_enum_update,
            mod_transactions_struct_data_member_rollback,
            mod_transactions_struct_data_member_log,
            struct_table_member_uniques: table_member_uniques,
            ..
        } = namer;

        let update_var = Ident::new("update", Span::call_site());
        let update_name = &self.alias;

        // Generate the table access to primary, and all associated!
        let assoc_brw_muts = (0..groups.assoc.len()).map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(let #name = unsafe { self.#table_member_columns.#name.assoc_brw_mut(index) } )
        });
        let table_access = quote! {
            let #pulpit_path::column::Entry { index, data: #name_primary_column } = match self.#table_member_columns.#name_primary_column.brw_mut(key) {
                Ok(entry) => entry,
                Err(_) => return Err(#mod_update::#update_name::#mod_update_enum_error::#type_key_error),
            };
            #(#assoc_brw_muts;)*
        };

        // Pass borrow of all fields to the predicate (check if it will be valid)
        // needs to include new updated values
        let predicate_args =
            generate_update_predicate_access(groups, &self.fields, &update_var, namer);
        let predicate_checks = predicates.iter().map(|pred| {
            let pred = &pred.alias;
            quote! {
                if !#mod_predicates::#pred(#predicate_args) {
                    return Err(#mod_update::#update_name::#mod_update_enum_error::#pred);
                }
            }
        });

        let mut undo_prev_fields: Vec<Tokens<ExprMethodCall>> = Vec::new();
        let mut unique_updates: Vec<Tokens<ExprLet>> = Vec::new();
        for Unique { alias, field } in uniques
            .iter()
            .filter(|uniq| self.fields.contains(&uniq.field))
        {
            let field_index = groups.idents.get(field).unwrap();
            let from_data = match field_index {
                FieldIndex::Primary(_) => namer.name_primary_column.clone(),
                FieldIndex::Assoc { assoc_ind, .. } => namer.name_assoc_column(*assoc_ind),
            };

            let mutability = if field_index.is_imm() {
                quote!(imm_data)
            } else {
                quote!(mut_data)
            };

            unique_updates.push(quote!{
                let #alias = match self.#table_member_uniques.#field.replace(&update.#field, &#from_data.#mutability.#field, key) {
                    Ok(old_val) => old_val,
                    Err(_) => {
                        #(#undo_prev_fields;)*
                        return Err(#mod_update::#update_name::#mod_update_enum_error::#alias)
                    },
                }
            }.into());

            undo_prev_fields.push(
                quote! {
                    self.#table_member_uniques.#field.undo_replace(#alias, &update.#field, key)
                }
                .into(),
            )
        }

        let update_pairs = self.fields.iter().map(|field| {
            let field_index = groups.idents.get(field).unwrap();
            let name_id = match field_index {
                FieldIndex::Primary(_) => namer.name_primary_column.clone(),
                FieldIndex::Assoc { assoc_ind, .. } => namer.name_assoc_column(*assoc_ind),
            };

            (field, quote!(#name_id.mut_data.#field))
        });

        let commit_updates = if transactions {
            let updates = update_pairs.map(|(field, mut_access)| {
                quote! {
                    std::mem::swap(&mut #mut_access, &mut update.#field);
                }
            });
            quote! {
                let mut update = update;
                #(#updates;)*

                if !self.#table_member_transactions.#mod_transactions_struct_data_member_rollback {
                    self.#table_member_transactions.#mod_transactions_struct_data_member_log.push(#mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_update(key, #mod_transactions::#mod_transactions_enum_update::#update_name(update)));
                }
            }
        } else {
            let updates = update_pairs.map(|(field, mut_access)| {
                quote! {
                    *(&mut #mut_access) = update.#field
                }
            });
            quote! {
                #(#updates;)*
            }
        };

        quote! {
            pub fn #update_name(&mut self, #update_var: #mod_update::#update_name::#mod_update_struct_update, key: #type_key) -> Result<(), #mod_update::#update_name::#mod_update_enum_error> {
                #table_access
                #(#predicate_checks)*
                #(#unique_updates;)*
                #commit_updates
                Ok(())
            }
        }
        .into()
    }
}
