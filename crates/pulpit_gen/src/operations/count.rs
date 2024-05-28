use super::SingleOpFn;
use crate::namer::CodeNamer;
use quote::quote;

pub fn generate(namer: &CodeNamer) -> SingleOpFn {
    let CodeNamer {
        struct_window_method_count: method_count,
        struct_window,
        name_primary_column,
        struct_table_member_columns: table_member_columns,
        ..
    } = namer;

    SingleOpFn {
        op_impl: quote! {
            impl <'imm> #struct_window<'imm> {
                pub fn #method_count(&self) -> usize {
                    self.#table_member_columns.#name_primary_column.count()
                }
            }
        }
        .into(),
    }
}
