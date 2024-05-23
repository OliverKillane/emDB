use super::SingleOp;
use crate::namer::CodeNamer;
use quote::quote;

pub fn generate(namer: &CodeNamer, transactions: bool) -> SingleOp {
    let type_key_error = namer.type_key_error();
    let type_key = namer.type_key();
    let struct_window = namer.struct_window();
    let name_primary_column = namer.name_primary_column();
    let table_member_columns = namer.table_member_columns();
    let mod_transactions_enum_logitem = namer.mod_transactions_enum_logitem();
    let mod_transactions_enum_logitem_variant_delete = namer.mod_transactions_enum_logitem_variant_delete();
    let mod_transactions = namer.mod_transactions();
    let table_member_transactions = namer.table_member_transactions();
    let mod_transactions_struct_data_member_rollback = namer.mod_transactions_struct_data_member_rollback();
    let mod_transactions_struct_data_member_log = namer.mod_transactions_struct_data_member_log();
    let mod_delete = namer.mod_delete();
    let trait_delete = namer.trait_delete();

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
