use proc_macro2::Span;
use quote::quote;
use quote_debug::Tokens;
use std::collections::{HashMap, HashSet};
use syn::{
    ExprLet, ExprMethodCall, Ident, ImplItemFn, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemTrait,
    TraitItemFn, Variant,
};

use crate::{
    columns::PrimaryKind,
    groups::{FieldIndex, FieldName, Groups},
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
    let trait_update = namer.trait_update();
    let mod_update = namer.mod_update();
    let struct_window = namer.struct_window();

    let modules = updates
        .iter()
        .map(|update| update.generate_mod(groups, uniques, predicates, namer));
    let trait_fns = updates.iter().map(|update| update.generate_trait_fn(namer));
    let impl_fns = updates
        .iter()
        .map(|update| update.generate_trait_impl_fn(namer, groups, uniques, predicates));

    SingleOp {
        op_mod: quote! {
            pub mod #mod_update {
                #(#modules)*
            }
        }
        .into(),
        op_trait: quote! {
            pub trait #trait_update : Sized {
                #(#trait_fns)*
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> #trait_update for #struct_window<'imm> {
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
            unique_indexes
                .map(|(_, unique)| {
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

        let mod_update_enum_error = namer.mod_update_enum_error();
        let type_key_error = namer.type_key_error();
        let mod_update_struct_update = namer.mod_update_struct_update();
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

    fn generate_trait_fn(&self, namer: &CodeNamer) -> Tokens<TraitItemFn> {
        let mod_update = namer.mod_update();
        let mod_update_struct_update = namer.mod_update_struct_update();
        let mod_update_enum_error = namer.mod_update_enum_error();
        let type_key = namer.type_key();
        
        let this_update = &self.alias;
        let update_name = &self.alias;

        quote! {
            fn #update_name(&mut self, update: #mod_update::#this_update::#mod_update_struct_update, key: #type_key) -> Result<(), #mod_update::#this_update::#mod_update_enum_error>;
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
        let mod_update = namer.mod_update();
        let mod_update_struct_update = namer.mod_update_struct_update();
        let mod_update_enum_error = namer.mod_update_enum_error();
        let name_primary_column = namer.name_primary_column();
        let table_member_columns = namer.table_member_columns();
        let type_key = namer.type_key();
        let mod_predicates = namer.mod_predicates();
        let type_key_error = namer.type_key_error();
        let pulpit_path = namer.pulpit_path();
        let mod_transactions_enum_logitem = namer.mod_transactions_enum_logitem();
        let mod_transactions = namer.mod_transactions();
        let mod_transactions_enum_logitem_variant_update = namer.mod_transactions_enum_logitem_variant_update();
        let table_member_transactions = namer.table_member_transactions();
        let mod_transactions_enum_update = namer.mod_transactions_enum_update();
        let mod_transactions_struct_data_member_rollback = namer.mod_transactions_struct_data_member_rollback();
        let mod_transactions_struct_data_member_log = namer.mod_transactions_struct_data_member_log();

        let update_var = Ident::new("update", Span::call_site());
        let update_name = &self.alias;

        // Generate the table access to primary, and all associated!
        let assoc_brw_muts = (0..groups.assoc.len()).map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(let #name = unsafe { self.#table_member_columns.#name.brw_mut(index) } )
        });
        let table_access = quote! {
            let #pulpit_path::column::Entry { index, data: #name_primary_column } = match self.#table_member_columns.#name_primary_column.brw_mut(key) {
                Ok(entry) => entry,
                Err(e) => return Err(#mod_update::#update_name::#mod_update_enum_error::#type_key_error),
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

        let uniques_member = namer.table_member_uniques();
        let mut undo_prev_fields: Vec<Tokens<ExprMethodCall>> = Vec::new();
        let mut unique_updates: Vec<Tokens<ExprLet>> = Vec::new();
        for (field, Unique { alias }) in uniques
            .iter()
            .filter(|(field, _)| self.fields.contains(field))
        {
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
                let #alias = match self.#uniques_member.#field.replace(&update.#field, &#from_data.#mutability.#field, key) {
                    Ok(old_val) => old_val,
                    Err(_) => {
                        #(#undo_prev_fields;)*
                        return Err(#mod_update::#update_name::#mod_update_enum_error::#alias)
                    },
                }
            }.into());

            undo_prev_fields.push(
                quote! {
                    self.#uniques_member.#field.undo_replace(#alias, &update.#field, key)
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
                    *#mut_access = update.#field
                }
            });
            quote! {
                #(#updates;)*
            }
        };

        quote! {
            fn #update_name(&mut self, #update_var: #mod_update::#update_name::#mod_update_struct_update, key: #type_key) -> Result<(), #mod_update::#update_name::#mod_update_enum_error> {
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
