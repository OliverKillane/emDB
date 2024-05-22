use super::SingleOp;
use crate::{columns::PrimaryKind, groups::Groups, namer::CodeNamer};
use quote::quote;

pub fn generate<Primary: PrimaryKind>(groups: &Groups<Primary>, namer: &CodeNamer) -> SingleOp {
    let key_error = namer.type_key_error();
    let key_type = namer.type_key();
    let window_struct = namer.struct_window();

    let mut include_lifetime = true; // TODO: implement
    let lifetime = if include_lifetime {
        quote!(<'imm>)
    } else {
        quote!()
    };

    SingleOp {
        op_mod: quote! {
            pub mod get {
                /// TODO
                pub struct Get #lifetime {
                    
                }
            }
        }
        .into(),
        op_trait: quote! {
            pub trait Get #lifetime {
                fn get(&self, key: #key_type) -> Result<get::Get #lifetime, #key_error>;
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> Get #lifetime for #window_struct<'imm> {
                fn get(&self, key: #key_type) -> Result<get::Get #lifetime, #key_error> {
                    todo!()
                }
            }
        }
        .into(),
    }
}
