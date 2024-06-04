use super::{update::Update, SingleOp};
use crate::{groups::Groups, namer::CodeNamer};
use quote::quote;

pub fn generate(
    groups: &Groups,
    updates: &[Update],
    namer: &CodeNamer,
    deletions: bool,
    transactions: bool,
) -> SingleOp {
    let CodeNamer {
        struct_window,
        mod_transactions_struct_data,
        mod_transactions_enum_logitem,
        mod_transactions_enum_update,
        mod_transactions,
        mod_update,
        mod_update_struct_update,
        mod_transactions_enum_logitem_variant_update,
        mod_transactions_enum_logitem_variant_insert,
        mod_transactions_enum_logitem_variant_append,
        mod_transactions_enum_logitem_variant_delete,
        struct_table_member_transactions: table_member_transactions,
        mod_transactions_struct_data_member_log,
        mod_transactions_struct_data_member_rollback,
        struct_table_member_columns: table_member_columns,
        type_key,
        name_primary_column,
        struct_window_method_commit: method_commit,
        struct_window_method_abort: method_abort,
        struct_window_method_delete_hidden,
        struct_window_method_reverse_insert,
        struct_window_method_restore_hidden,
        ..
    } = namer;

    let updates_variants = updates.iter().map(
        |Update { fields: _, alias }| quote!(#alias(super::#mod_update::#alias::#mod_update_struct_update)),
    );

    let log_variants = if deletions {
        quote! {
            #mod_transactions_enum_logitem_variant_update(super::#type_key, #mod_transactions_enum_update),
            #mod_transactions_enum_logitem_variant_insert(super::#type_key),
            #mod_transactions_enum_logitem_variant_delete(super::#type_key),
        }
    } else {
        quote! {
            #mod_transactions_enum_logitem_variant_update(super::#type_key, #mod_transactions_enum_update),
            #mod_transactions_enum_logitem_variant_append,
        }
    };

    let abort_update = updates.iter().map(|Update { fields: _, alias }| {
        quote! {
            #mod_transactions::#mod_transactions_enum_update::#alias(update) => {
                self.#alias(update, key).unwrap();
            }
        }
    });
    let update_rollback_case = quote! {#mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_update(key, update) => {
        match update {
            #(#abort_update,)*
        }
    }};

    let op_impl = if deletions {
        quote! {
            impl <'imm> #struct_window<'imm> {
                /// Commit all current changes
                /// - Requires concretely applying deletions (which until commit 
                ///   or abort simply hide keys from the table)
                pub fn #method_commit(&mut self) {
                    debug_assert!(!self.#table_member_transactions.#mod_transactions_struct_data_member_rollback);
                    while let Some(entry) = self.#table_member_transactions.#mod_transactions_struct_data_member_log.pop() {
                        match entry {
                            #mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_delete(key) => {
                                self.#struct_window_method_restore_hidden(key);
                            },
                            _ => (),
                        }
                    }
                }

                /// Undo the transactions applied since the last commit
                /// - Requires re-applying all updates, deleting inserts and undoing deletes 
                ///   (deletes' keys are actually just hidden until commit or abort)
                pub fn #method_abort(&mut self) {
                    self.#table_member_transactions.#mod_transactions_struct_data_member_rollback = true;
                    while let Some(entry) = self.#table_member_transactions.#mod_transactions_struct_data_member_log.pop() {
                        match entry {
                            #mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_delete(key) => {
                                self.#struct_window_method_delete_hidden(key);
                            },
                            #mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_insert(key) => {
                                self.#struct_window_method_reverse_insert(key);
                            },
                            #update_rollback_case
                        }
                    }
                    self.#table_member_transactions.#mod_transactions_struct_data_member_rollback = false;
                }
            }
        }
        .into()
    } else {
        let assoc_cols = (0..groups.assoc.len()).map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(self.#table_member_columns.#name.assoc_unppend())
        });

        quote! {
            impl <'imm> #struct_window<'imm> {
                /// Commit all current changes
                /// - Clears the rollback log
                pub fn #method_commit(&mut self) {
                    debug_assert!(!self.#table_member_transactions.#mod_transactions_struct_data_member_rollback);
                    self.#table_member_transactions.#mod_transactions_struct_data_member_log.clear()
                }

                /// Undo the transactions applied since the last commit
                /// - Requires re-applying all updates, deleting inserts and undoing deletes
                ///   (deletes' keys are actually just hidden until commit or abort)
                pub fn #method_abort(&mut self) {
                    self.#table_member_transactions.#mod_transactions_struct_data_member_rollback = true;
                    while let Some(entry) = self.#table_member_transactions.#mod_transactions_struct_data_member_log.pop() {
                        match entry {
                            #mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_append => {
                                unsafe{
                                    self.#table_member_columns.#name_primary_column.unppend();
                                    #(#assoc_cols;)*
                                }
                            },
                            #update_rollback_case
                        }
                    }
                    self.#table_member_transactions.#mod_transactions_struct_data_member_rollback = false;
                }
            }
        }
        .into()
    };

    SingleOp {
        op_mod: quote! {
            mod #mod_transactions {
                pub enum #mod_transactions_enum_update {
                    #(#updates_variants,)*
                }
                pub enum #mod_transactions_enum_logitem {
                    #log_variants
                }
                pub struct #mod_transactions_struct_data {
                    pub #mod_transactions_struct_data_member_log: Vec<#mod_transactions_enum_logitem>,
                    pub #mod_transactions_struct_data_member_rollback: bool,
                }
                impl #mod_transactions_struct_data {
                    pub fn new() -> Self {
                        Self {
                            #mod_transactions_struct_data_member_log: Vec::new(),
                            #mod_transactions_struct_data_member_rollback: false,
                        }
                    }
                }
            }
        }
        .into(),
        op_impl,
    }
}
