use super::SingleOp;
use crate::v2::{columns::PrimaryKind, namer::CodeNamer};
use quote::quote;

pub fn generate(namer: &CodeNamer) -> SingleOp {
    let borrow_struct_name = namer.mod_borrow_struct_borrow();
    let key_error = namer.type_key_error();
    let key_type = namer.type_key();
    let window_struct = namer.struct_window();

    SingleOp {
        op_mod: quote! {
            mod delete {}
        }.into(),
        op_trait: quote! {
            pub trait Delete {
                fn delete(&'brw self, key: #key_type) -> Result<#borrow_struct_name<'brw>, #key_error>;
            }
        }.into(),
        op_impl: quote! {
            impl <'imm> Delete for #window_struct<'imm> {
                fn borrow(&'brw self, key: #key_type) -> Result<(), #key_error> {
                    todo!()
                }
            }
        }.into(),
    }
}
