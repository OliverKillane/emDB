use super::SingleOp;
use crate::{groups::Groups, namer::CodeNamer, uniques::Unique};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(
    groups: &Groups,
    uniques: &[Unique],
    namer: &CodeNamer,
    op_attrs: &TokenStream,
) -> SingleOp {
    let CodeNamer {
        struct_window,
        struct_table_member_uniques,
        type_key,
        mod_unique,
        mod_unique_struct_notfound,
        ..
    } = namer;

    let unique_methods = uniques.iter().map(|Unique { alias, field }| {
        let ty = groups.get_typefield(field).unwrap();
        quote!{
            #op_attrs
            pub fn #alias(&self, value: &#ty) -> Result<#type_key, #mod_unique::#mod_unique_struct_notfound> {
                match self.#struct_table_member_uniques.#field.lookup(value) {
                    Ok(k) => Ok(k),
                    Err(_) => Err(#mod_unique::#mod_unique_struct_notfound),
                }
            }
        }
    });

    SingleOp {
        op_mod: quote! {
            pub mod #mod_unique {
                #[derive(Debug)]
                pub struct #mod_unique_struct_notfound;
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> #struct_window<'imm> {
                #(#unique_methods)*
            }
        }
        .into(),
    }
}
