use super::SingleOp;
use crate::{columns::PrimaryKind, groups::Groups, namer::CodeNamer};
use quote::quote;

pub fn generate<Primary: PrimaryKind>(groups: &Groups<Primary>, namer: &CodeNamer) -> SingleOp {
    let key_error = namer.type_key_error();
    let key_type = namer.type_key();
    let window_struct = namer.struct_window();

    let struct_fields = groups.idents.iter().map(|(field_name, field_index)| {
        let field_ty = groups.get_type(field_index).unwrap();
        quote!(#field_name: &'brw #field_ty)
    });

    SingleOp {
        op_mod: quote! {
            pub mod borrow {
                pub struct Borrow<'brw> {
                    #(#struct_fields),*
                }
            }
        }
        .into(),
        op_trait: quote! {
            pub trait Borrow {
                fn borrow<'brw>(&'brw self, key: #key_type) -> Result<borrow::Borrow<'brw>, #key_error>;
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> Borrow for #window_struct<'imm> {
                fn borrow<'brw>(&'brw self, key: #key_type) -> Result<borrow::Borrow<'brw>, #key_error> {
                    todo!()
                }
            }
        }
        .into(),
    }
}
