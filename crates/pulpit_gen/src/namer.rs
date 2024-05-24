use proc_macro2::Span;
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, Lifetime, Path};

pub struct CodeNamer {
    pub lifetime_imm: Tokens<Lifetime>,
    pub mod_columns: Ident,
    pub name_primary_column: Ident,
    pub mod_columns_struct_imm: Ident,
    pub mod_columns_struct_mut: Ident,
    pub mod_columns_struct_imm_unpacked: Ident,
    pub mod_columns_fn_imm_unpack: Ident,
    pub pulpit_path: Tokens<Path>,
    pub struct_column_holder: Ident,
    pub struct_window_holder: Ident,
    pub mod_update_struct_update: Ident,
    pub mod_update_enum_error: Ident,
    pub type_key_error: Ident,
    pub table_member_columns: Ident,
    pub type_key: Ident,
    pub mod_predicates: Ident,
    pub struct_uniques_holder: Ident,
    pub table_member_uniques: Ident,
    pub table_member_transactions: Ident,
    pub mod_transactions: Ident,
    pub mod_transactions_enum_logitem: Ident,
    pub mod_transactions_enum_update: Ident,
    pub mod_update: Ident,
    pub mod_borrow_struct_borrow: Ident,
    pub struct_window: Ident,
    pub struct_table: Ident,
    pub mod_borrow: Ident,
    pub mod_get: Ident,
    pub mod_get_struct_get: Ident,
    pub mod_insert: Ident,
    pub mod_insert_struct_insert: Ident,
    pub mod_insert_enum_error: Ident,
    pub struct_unique: Ident,
    pub mod_transactions_struct_data: Ident,
    pub mod_transactions_struct_data_member_log: Ident,
    pub mod_transactions_struct_data_member_rollback: Ident,
    pub mod_transactions_enum_logitem_variant_update: Ident,
    pub mod_transactions_enum_logitem_variant_insert: Ident,
    pub mod_transactions_enum_logitem_variant_append: Ident,
    pub mod_transactions_enum_logitem_variant_delete: Ident,
    pub mod_delete: Ident,
    pub method_commit: Ident,
    pub method_abort: Ident,
    pub method_get: Ident,
    pub method_borrow: Ident,
    pub method_insert: Ident,
    pub method_delete: Ident,
}

fn new_id(id: &str) -> Ident {
    Ident::new(id, Span::call_site())
}

impl Default for CodeNamer {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeNamer {
    pub fn new() -> Self {
        Self {
            lifetime_imm: quote! {'imm}.into(),
            mod_columns: new_id("column_types"),
            name_primary_column: new_id("primary"),
            mod_columns_struct_imm: new_id("Imm"),
            mod_columns_struct_mut: new_id("Mut"),
            mod_columns_struct_imm_unpacked: new_id("ImmUnpack"),
            mod_columns_fn_imm_unpack: new_id("imm_unpack"),
            pulpit_path: quote!(pulpit).into(),
            struct_column_holder: new_id("ColumnHolder"),
            struct_window_holder: new_id("WindowHolder"),
            mod_update_struct_update: new_id("Update"),
            mod_update_enum_error: new_id("UpdateError"),
            type_key_error: new_id("KeyError"),
            table_member_columns: new_id("columns"),
            type_key: new_id("Key"),
            mod_predicates: new_id("predicates"),
            struct_uniques_holder: new_id("Uniques"),
            table_member_uniques: new_id("uniques"),
            table_member_transactions: new_id("transactions"),
            mod_transactions: new_id("transactions"),
            mod_transactions_enum_logitem: new_id("LogItem"),
            mod_transactions_enum_update: new_id("Updates"),
            mod_update: new_id("updates"),
            mod_borrow_struct_borrow: new_id("Borrows"),
            struct_window: new_id("Window"),
            struct_table: new_id("Table"),
            mod_borrow: new_id("borrows"),
            mod_get: new_id("get"),
            mod_get_struct_get: new_id("Get"),
            mod_insert: new_id("insert"),
            mod_insert_struct_insert: new_id("Insert"),
            mod_insert_enum_error: new_id("Error"),
            struct_unique: new_id("Uniques"),
            mod_transactions_struct_data: new_id("Data"),
            mod_transactions_struct_data_member_log: new_id("log"),
            mod_transactions_struct_data_member_rollback: new_id("rollback"),
            mod_transactions_enum_logitem_variant_update: new_id("Update"),
            mod_transactions_enum_logitem_variant_insert: new_id("Insert"),
            mod_transactions_enum_logitem_variant_append: new_id("Append"),
            mod_transactions_enum_logitem_variant_delete: new_id("Delete"),
            mod_delete: new_id("delete"),
            method_commit: new_id("commit"),
            method_abort: new_id("abort"),
            method_get: new_id("get"),
            method_borrow: new_id("borrow"),
            method_insert: new_id("insert"),
            method_delete: new_id("delete"),
        }
    }
    pub fn name_assoc_column(&self, assoc_ind: usize) -> Ident {
        Ident::new(&format!("assoc_{assoc_ind}"), Span::call_site())
    }
}
