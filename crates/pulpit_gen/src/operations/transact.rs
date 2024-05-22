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

    SingleOp {
        op_mod: quote! {
            mod #trans_mod {
                ///TODO
                pub enum #update_enum {

                }
                
                /// TODO
                pub enum #log_item {
                    
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
        op_impl: quote! {
            impl <'imm> Transact for #window_struct<'imm> {
                fn commit(&mut self) { todo!() }
                fn abort(&mut self) { todo!() }
            }
        }
        .into(),
    }
}
