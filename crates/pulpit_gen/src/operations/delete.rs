use super::SingleOp;
use crate::{columns::PrimaryKind, groups::Groups, namer::CodeNamer};
use quote::quote;

pub fn generate<Primary: PrimaryKind>(namer: &CodeNamer, groups: &Groups<Primary>,) -> SingleOp {
    let CodeNamer {
        type_key_error,
        type_key,
        struct_window,
        name_primary_column,
        table_member_columns,
        mod_transactions_enum_logitem,
        mod_transactions_enum_logitem_variant_delete,
        mod_transactions,
        table_member_transactions,
        mod_transactions_struct_data_member_rollback,
        mod_transactions_struct_data_member_log,
        mod_delete,
        method_delete,
        pulpit_path,
        ..
    } = namer;

    assert!(Primary::DELETIONS);

    let op_impl = if Primary::TRANSACTIONS {
        let transactional = quote! {
            if !self.#table_member_transactions.#mod_transactions_struct_data_member_rollback {
                self.#table_member_transactions.#mod_transactions_struct_data_member_log.push(#mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_delete(key));
            }
        };

        quote! {
            impl <'imm> #struct_window<'imm> {
                pub fn #method_delete(&mut self, key: #type_key) -> Result<(), #type_key_error> {
                    match self.#table_member_columns.#name_primary_column.hide(key) {
                        Ok(()) => (),
                        Err(_) => return Err(#type_key_error),
                    }
                    #transactional
                    Ok(())
                }
            }
        }
        .into()
    } else {
        let assoc_cols = (0..groups.assoc.len()).map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(self.#table_member_columns.#name.pull(index))
        });

        quote!{
            impl <'imm> #struct_window<'imm> {
                pub fn #method_delete(&mut self, key: #type_key) -> Result<(), #type_key_error> {
                    match self.#table_member_columns.#name_primary_column.pull(key) {
                        Ok(#pulpit_path::column::Entry{ index, data:_ }) => {
                            unsafe {
                                #(#assoc_cols;)*
                            }
                            Ok(())
                        },
                        Err(_) => Err(#type_key_error),
                    }
                }
            }
        }.into()
    };

    SingleOp {
        op_mod: quote! { mod #mod_delete {} }.into(),
        op_impl,
    }
}
