use super::{update::Update, SingleOp};
use crate::{columns::PrimaryKind, groups::Groups, namer::CodeNamer};
use quote::quote;

pub fn generate<Primary: PrimaryKind>(
    groups: &Groups<Primary>,
    updates: &[Update],
    namer: &CodeNamer,
) -> SingleOp {
    let window_struct = namer.struct_window();

    // TODO: naming
    let data_struct = namer.mod_transactions_struct_data();
    let log_item = namer.mod_transactions_enum_logitem();
    let update_enum = namer.mod_transactions_enum_update();

    let trans_mod = namer.mod_transactions();

    let rollback_name = namer.mod_transactions_struct_data_member_rollback();
    let log_name = namer.mod_transactions_struct_data_member_log();

    let updates_mod = namer.mod_update();
    let update_struct = namer.mod_update_struct_update();

    let updates_variants = updates.iter().map(
        |Update { fields: _, alias }| quote!(#alias(super::#updates_mod::#alias::#update_struct)),
    );

    let variant_update = namer.mod_transactions_enum_logitem_variant_update();
    let variant_insert = namer.mod_transactions_enum_logitem_variant_insert();
    let variant_append = namer.mod_transactions_enum_logitem_variant_append();
    let variant_delete = namer.mod_transactions_enum_logitem_variant_delete();

    let key_type = namer.type_key();

    let log_variants = if Primary::DELETIONS {
        quote! {
            #variant_update(super::#key_type, #update_enum),
            #variant_insert(super::#key_type),
            #variant_delete(super::#key_type),
        }
    } else {
        quote! {
            #variant_update(#update_enum),
            #variant_append,
        }
    };

    let trans_member = namer.table_member_transactions();
    let log_name = namer.mod_transactions_struct_data_member_log();
    let rollback_name = namer.mod_transactions_struct_data_member_rollback();

    let col_member = namer.table_member_columns();

    let update_trait = namer.trait_update();
    let abort_update = updates.iter().map(|Update { fields: _, alias }| {
        quote! {
            #trans_mod::#update_enum::#alias(update) => {
                <Self as #update_trait>::#alias(self, update, key).unwrap();
            }
        }
    });
    let update_rollback_case = quote! {#trans_mod::#log_item::#variant_update(key, update) => {
        match update {
            #(#abort_update,)*
        }
    }};

    let primary_name = namer.name_primary_column();

    let op_impl = if Primary::DELETIONS {
        let pulpit_path = namer.pulpit_path();
        let assoc_cols = (0..groups.assoc.len()).map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(self.#col_member.#name.pull(index))
        });
        let assoc_cols_abrt_del = assoc_cols.clone();

        quote! {
            impl <'imm> Transact for #window_struct<'imm> {
                /// Commit all current changes
                /// - Requires concretely applying deletions (which until commit 
                ///   or abort simply hide keys from the table)
                fn commit(&mut self) {
                    debug_assert!(!self.#trans_member.#rollback_name);
                    while let Some(entry) = self.#trans_member.#log_name.pop() {
                        match entry {
                            #trans_mod::#log_item::#variant_delete(key) => {
                                let #pulpit_path::column::Entry{ index, data:_ } = self.#col_member.#primary_name.pull(key).unwrap();
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
                    self.#trans_member.#rollback_name = true;
                    while let Some(entry) = self.#trans_member.#log_name.pop() {
                        match entry {
                            #trans_mod::#log_item::#variant_delete(key) => {
                                self.#col_member.#primary_name.reveal(key).unwrap();
                            },
                            #trans_mod::#log_item::#variant_insert(key) => {
                                let #pulpit_path::column::Entry{ index, data:_ } = self.#col_member.#primary_name.pull(key).unwrap();
                                unsafe {
                                    #(#assoc_cols_abrt_del;)*
                                }
                            },
                            #update_rollback_case
                        }
                    }
                    self.#trans_member.#rollback_name = false;
                }
            }
        }
        .into()
    } else {
        let assoc_cols = (0..groups.assoc.len()).map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(self.#col_member.#name.unppend())
        });

        quote! {
            impl <'imm> Transact for #window_struct<'imm> {
                /// Commit all current changes
                /// - Clears the rollback log
                fn commit(&mut self) {
                    debug_assert!(!self.#trans_member.#rollback_name);
                    self.#trans_member.#log_name.clear()
                }

                /// Undo the transactions applied since the last commit
                /// - Requires re-applying all updates, deleting inserts and undoing deletes
                ///   (deletes' keys are actually just hidden until commit or abort)
                fn abort(&mut self) {
                    self.#trans_member.#rollback_name = true;
                    while let Some(entry) = self.#trans_member.#log_name.pop() {
                        match entry {
                            #trans_mod::#log_item::#variant_append => {
                                unsafe{
                                    self.#trans_member.#primary_name.unppend();
                                    #(#assoc_cols;)*
                                }
                            },
                            #update_rollback_case
                        }
                    }
                    self.#trans_member.#rollback_name = false;
                }
            }
        }
        .into()
    };

    SingleOp {
        op_mod: quote! {
            mod #trans_mod {
                ///TODO
                pub enum #update_enum {
                    #(#updates_variants,)*
                }

                /// TODO
                pub enum #log_item {
                    #log_variants
                }

                pub struct #data_struct {
                    pub #log_name: Vec<#log_item>,
                    pub #rollback_name: bool,
                }

                impl #data_struct {
                    pub fn new() -> Self {
                        Self {
                            #log_name: Vec::new(),
                            #rollback_name: false,
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
