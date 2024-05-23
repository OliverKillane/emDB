use super::SingleOp;
use crate::namer::CodeNamer;
use quote::quote;

pub fn generate(namer: &CodeNamer, transactions: bool) -> SingleOp {
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
        trait_delete,
        ..
    } = namer;

    let transactional = if transactions {
        quote! {
            if !self.#table_member_transactions.#mod_transactions_struct_data_member_rollback {
                self.#table_member_transactions.#mod_transactions_struct_data_member_log.push(#mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_delete(key));
            }
        }
    } else {
        quote!()
    };

    SingleOp {
        op_mod: quote! {
            mod #mod_delete {}
        }
        .into(),
        op_trait: quote! {
            pub trait #trait_delete {
                /// Remove the key from the table and drop the contents.
                fn delete(&mut self, key: #type_key) -> Result<(), #type_key_error>;
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> #trait_delete for #struct_window<'imm> {
                fn delete(&mut self, key: #type_key) -> Result<(), #type_key_error> {
                    match self.#table_member_columns.#name_primary_column.hide(key) {
                        Ok(()) => (),
                        Err(e) => return Err(#type_key_error),
                    }
                    #transactional
                    Ok(())
                }
            }
        }
        .into(),
    }
}
