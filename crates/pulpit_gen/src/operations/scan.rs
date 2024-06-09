use proc_macro2::TokenStream;
use quote::quote;

use crate::namer::CodeNamer;

use super::SingleOpFn;

pub fn generate(namer: &CodeNamer, op_attrs: &TokenStream) -> SingleOpFn {
    let CodeNamer {
        struct_window_method_scan_brw,
        struct_window_method_scan_get,
        type_key,
        struct_window,
        name_primary_column,
        lifetime_imm,
        struct_table_member_columns: table_member_columns,
        ..
    } = namer;

    SingleOpFn {
        op_impl: quote! {
            impl <#lifetime_imm> #struct_window<#lifetime_imm> {
                #op_attrs
                pub fn #struct_window_method_scan_brw(&self) -> impl Iterator<Item = #type_key> + '_ {
                    self.#table_member_columns.#name_primary_column.scan_brw()
                }

                #op_attrs
                pub fn #struct_window_method_scan_get(&self) -> impl Iterator<Item = #type_key> + '_ {
                    self.#table_member_columns.#name_primary_column.scan_get()
                }
            }
        }
        .into(),
    }
}
