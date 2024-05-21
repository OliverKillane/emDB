use super::SingleOp;
use crate::{
    columns::{Groups, PrimaryKind},
    namer::CodeNamer,
};
use quote::quote;

pub fn generate<Primary: PrimaryKind>(groups: &Groups<Primary>, namer: &CodeNamer) -> SingleOp {
    let key_error = namer.type_key_error();
    let key_type = namer.type_key();
    let window_struct = namer.struct_window();

    SingleOp {
        op_mod: quote! {
            pub mod insert {
                pub struct Insert {
                    unimplemented!()
                }

                pub enum Error {
                    unimplemented!()
                }
            }
        }
        .into(),
        op_trait: quote! {
            pub trait Insert {
                fn get(&self, insert: insert::Insert) -> Result<#key_type, insert::Error>;
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> Insert for #window_struct<'imm> {
                fn get(&self, insert: insert::Insert) -> Result<#key_type, insert::Error> {
                    todo!()
                }
            }
        }
        .into(),
    }
}
