use super::SingleOp;
use crate::{columns::PrimaryKind, groups::Groups, namer::CodeNamer};
use quote::quote;

pub fn generate<Primary: PrimaryKind>(groups: &Groups<Primary>, namer: &CodeNamer) -> SingleOp {
    let key_error = namer.type_key_error();
    let key_type = namer.type_key();
    let window_struct = namer.struct_window();

    SingleOp {
        op_mod: quote! {
            pub mod insert {
                /// TODO
                pub struct Insert {
                }

                /// TODO
                #[derive(Debug)]
                pub enum Error {
                }
            }
        }
        .into(),
        op_trait: quote! {
            pub trait Insert {
                fn insert(&self, insert: insert::Insert) -> Result<#key_type, insert::Error>;
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> Insert for #window_struct<'imm> {
                fn insert(&self, insert: insert::Insert) -> Result<#key_type, insert::Error> {
                    todo!()
                }
            }
        }
        .into(),
    }
}
