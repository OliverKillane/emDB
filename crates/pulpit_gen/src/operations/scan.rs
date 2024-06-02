use quote::quote;

use crate::namer::CodeNamer;

use super::SingleOpFn;

pub fn generate(namer: &CodeNamer) -> SingleOpFn {
    let CodeNamer {
        struct_window_method_scan: method_scan,
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
                pub fn #method_scan(&self) -> impl Iterator<Item = #type_key> + '_ {
                    self.#table_member_columns.#name_primary_column.scan()
                }
            }
        }
        .into(),
    }
}
