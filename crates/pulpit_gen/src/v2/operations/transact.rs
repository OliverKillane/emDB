use super::{update::Update, SingleOp};
use crate::v2::{
    columns::{Groups, PrimaryKind},
    namer::CodeNamer,
};
use quote::quote;

pub fn generate<Primary: PrimaryKind>(
    groups: &Groups<Primary>,
    updates: &[Update],
    namer: &CodeNamer,
) -> SingleOp {
    let window_struct = namer.struct_window();

    // TODO: naming

    SingleOp {
        op_mod: quote! {
            pub mod transaction {
                pub enum LogItem {
                    unimplemented!()
                }

                pub struct Data {
                    log: Vec<LogItem>,
                    rollback: bool,
                }

                impl Data {
                    pub fn new() -> Self {
                        Self {
                            log: Vec::new(),
                            rollback: false,
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
