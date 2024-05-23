use super::{update::Update, SingleOp};
use crate::{columns::PrimaryKind, groups::Groups, namer::CodeNamer};
use quote::quote;

pub fn generate<Primary: PrimaryKind>(
    groups: &Groups<Primary>,
    updates: &[Update],
    namer: &CodeNamer,
) -> SingleOp {
    let struct_window = namer.struct_window();
    let mod_transactions_struct_data = namer.mod_transactions_struct_data();
    let mod_transactions_enum_logitem = namer.mod_transactions_enum_logitem();
    let mod_transactions_enum_update = namer.mod_transactions_enum_update();
    let mod_transactions = namer.mod_transactions();
    let mod_update = namer.mod_update();
    let mod_update_struct_update = namer.mod_update_struct_update();
    let mod_transactions_enum_logitem_variant_update = namer.mod_transactions_enum_logitem_variant_update();
    let mod_transactions_enum_logitem_variant_insert = namer.mod_transactions_enum_logitem_variant_insert();
    let mod_transactions_enum_logitem_variant_append = namer.mod_transactions_enum_logitem_variant_append();
    let mod_transactions_enum_logitem_variant_delete = namer.mod_transactions_enum_logitem_variant_delete();
    let table_member_transactions = namer.table_member_transactions();
    let mod_transactions_struct_data_member_log = namer.mod_transactions_struct_data_member_log();
    let mod_transactions_struct_data_member_rollback = namer.mod_transactions_struct_data_member_rollback();
    let table_member_columns = namer.table_member_columns();
    let trait_update = namer.trait_update();
    let type_key = namer.type_key();
    let name_primary_column = namer.name_primary_column();
    let pulpit_path = namer.pulpit_path();

    let updates_variants = updates.iter().map(
        |Update { fields: _, alias }| quote!(#alias(super::#mod_update::#alias::#mod_update_struct_update)),
    );

    let log_variants = if Primary::DELETIONS {
        quote! {
            #mod_transactions_enum_logitem_variant_update(super::#type_key, #mod_transactions_enum_update),
            #mod_transactions_enum_logitem_variant_insert(super::#type_key),
            #mod_transactions_enum_logitem_variant_delete(super::#type_key),
        }
    } else {
        quote! {
            #mod_transactions_enum_logitem_variant_update(#mod_transactions_enum_update),
            #mod_transactions_enum_logitem_variant_append,
        }
    };

    let abort_update = updates.iter().map(|Update { fields: _, alias }| {
        quote! {
            #mod_transactions::#mod_transactions_enum_update::#alias(update) => {
                <Self as #trait_update>::#alias(self, update, key).unwrap();
            }
        }
    });
    let update_rollback_case = quote! {#mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_update(key, update) => {
        match update {
            #(#abort_update,)*
        }
    }};

    let op_impl = if Primary::DELETIONS {
        let assoc_cols = (0..groups.assoc.len()).map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(self.#table_member_columns.#name.pull(index))
        });
        let assoc_cols_abrt_del = assoc_cols.clone();

        quote! {
            impl <'imm> Transact for #struct_window<'imm> {
                /// Commit all current changes
                /// - Requires concretely applying deletions (which until commit 
                ///   or abort simply hide keys from the table)
                fn commit(&mut self) {
                    debug_assert!(!self.#table_member_transactions.#mod_transactions_struct_data_member_rollback);
                    while let Some(entry) = self.#table_member_transactions.#mod_transactions_struct_data_member_log.pop() {
                        match entry {
                            #mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_delete(key) => {
                                let #pulpit_path::column::Entry{ index, data:_ } = self.#table_member_columns.#name_primary_column.pull(key).unwrap();
                                unsafe {
                                    #(#assoc_cols;)*
                                }
                            },
                            _ => (),
                        }
                    }
                }

                /// Undo the transactions applied since the last commit
                /// - Requires re-applying all updates, deleting inserts and undoing deletes 
                ///   (deletes' keys are actually just hidden until commit or abort)
                fn abort(&mut self) {
                    self.#table_member_transactions.#mod_transactions_struct_data_member_rollback = true;
                    while let Some(entry) = self.#table_member_transactions.#mod_transactions_struct_data_member_log.pop() {
                        match entry {
                            #mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_delete(key) => {
                                self.#table_member_columns.#name_primary_column.reveal(key).unwrap();
                            },
                            #mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_insert(key) => {
                                let #pulpit_path::column::Entry{ index, data:_ } = self.#table_member_columns.#name_primary_column.pull(key).unwrap();
                                unsafe {
                                    #(#assoc_cols_abrt_del;)*
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
    } else {
        let assoc_cols = (0..groups.assoc.len()).map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(self.#table_member_columns.#name.unppend())
        });

        quote! {
            impl <'imm> Transact for #struct_window<'imm> {
                /// Commit all current changes
                /// - Clears the rollback log
                fn commit(&mut self) {
                    debug_assert!(!self.#table_member_transactions.#mod_transactions_struct_data_member_rollback);
                    self.#table_member_transactions.#mod_transactions_struct_data_member_log.clear()
                }

                /// Undo the transactions applied since the last commit
                /// - Requires re-applying all updates, deleting inserts and undoing deletes
                ///   (deletes' keys are actually just hidden until commit or abort)
                fn abort(&mut self) {
                    self.#table_member_transactions.#mod_transactions_struct_data_member_rollback = true;
                    while let Some(entry) = self.#table_member_transactions.#mod_transactions_struct_data_member_log.pop() {
                        match entry {
                            #mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_append => {
                                unsafe{
                                    self.#table_member_transactions.#name_primary_column.unppend();
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
                ///TODO
                pub enum #mod_transactions_enum_update {
                    #(#updates_variants,)*
                }

                /// TODO
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
        op_trait: quote! {
            pub trait Transact {
                fn commit(&mut self);
                fn abort(&mut self);
            }
        }
        .into(),
        op_impl,
    }
}
