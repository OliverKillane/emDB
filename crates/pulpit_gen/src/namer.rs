use proc_macro2::Span;
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, Lifetime, Path};

pub struct CodeNamer;

impl CodeNamer {
    pub fn lifetime_imm(&self) -> Tokens<Lifetime> {
        quote! {'imm}.into()
    }

    pub fn mod_columns(&self) -> Ident {
        Ident::new("column_types", Span::call_site())
    }

    pub fn name_primary_column(&self) -> Ident {
        Ident::new("primary", Span::call_site())
    }

    pub fn name_assoc_column(&self, assoc_ind: usize) -> Ident {
        Ident::new(&format!("assoc_{assoc_ind}"), Span::call_site())
    }

    pub fn mod_columns_struct_imm(&self) -> Ident {
        Ident::new("Imm", Span::call_site())
    }

    pub fn mod_columns_struct_mut(&self) -> Ident {
        Ident::new("Mut", Span::call_site())
    }

    pub fn mod_columns_struct_imm_unpacked(&self) -> Ident {
        Ident::new("ImmUnpack", Span::call_site())
    }

    pub fn mod_columns_fn_imm_unpack(&self) -> Ident {
        Ident::new("imm_unpack", Span::call_site())
    }

    pub fn pulpit_path(&self) -> Tokens<Path> {
        quote! {pulpit}.into()
    }

    pub fn struct_column_holder(&self) -> Ident {
        Ident::new("ColumnHolder", Span::call_site())
    }

    pub fn struct_window_holder(&self) -> Ident {
        Ident::new("WindowHolder", Span::call_site())
    }

    pub fn mod_update_struct_update(&self) -> Ident {
        Ident::new("Update", Span::call_site())
    }

    pub fn mod_update_enum_error(&self) -> Ident {
        Ident::new("UpdateError", Span::call_site())
    }

    pub fn type_key_error(&self) -> Ident {
        Ident::new("KeyError", Span::call_site())
    }

    pub fn table_member_columns(&self) -> Ident {
        Ident::new("columns", Span::call_site())
    }

    pub fn type_key(&self) -> Ident {
        Ident::new("Key", Span::call_site())
    }

    pub fn mod_predicates(&self) -> Ident {
        Ident::new("predicates", Span::call_site())
    }

    pub fn struct_uniques_holder(&self) -> Ident {
        Ident::new("Uniques", Span::call_site())
    }

    pub fn table_member_uniques(&self) -> Ident {
        Ident::new("uniques", Span::call_site())
    }

    pub fn table_member_transactions(&self) -> Ident {
        Ident::new("transactions", Span::call_site())
    }

    pub fn mod_transactions(&self) -> Ident {
        Ident::new("transactions", Span::call_site())
    }

    pub fn mod_transactions_enum_logitem(&self) -> Ident {
        Ident::new("LogItem", Span::call_site())
    }

    pub fn mod_transactions_enum_update(&self) -> Ident {
        Ident::new("Updates", Span::call_site())
    }

    pub fn trait_update(&self) -> Ident {
        Ident::new("Update", Span::call_site())
    }

    pub fn mod_update(&self) -> Ident {
        Ident::new("updates", Span::call_site())
    }

    pub fn mod_borrow_struct_borrow(&self) -> Ident {
        Ident::new("Borrows", Span::call_site())
    }

    pub fn trait_borrow(&self) -> Ident {
        Ident::new("Borrow", Span::call_site())
    }

    pub fn struct_window(&self) -> Ident {
        Ident::new("Window", Span::call_site())
    }

    pub fn struct_table(&self) -> Ident {
        Ident::new("Table", Span::call_site())
    }

    pub fn mod_borrow(&self) -> Ident {
        Ident::new("borrows", Span::call_site())
    }



    pub fn mod_get(&self) -> Ident {
        Ident::new("get", Span::call_site())
    }

    pub fn mod_get_struct_get(&self) -> Ident {
        Ident::new("Get", Span::call_site())
    }

    pub fn trait_get(&self) -> Ident {
        Ident::new("Get", Span::call_site())
    }

    pub fn mod_insert(&self) -> Ident {
        Ident::new("insert", Span::call_site())
    }

    pub fn mod_insert_struct_insert(&self) -> Ident {
        Ident::new("Insert", Span::call_site())
    }

    pub fn mod_insert_enum_error(&self) -> Ident {
        Ident::new("Error", Span::call_site())
    }

    pub fn trait_insert(&self) -> Ident {
        Ident::new("Insert", Span::call_site())
    }

    pub fn struct_unique(&self) -> Ident {
        Ident::new("Uniques", Span::call_site())
    }

    pub fn mod_transactions_struct_data(&self) -> Ident {
        Ident::new("Data", Span::call_site())
    }

    pub fn mod_transactions_struct_data_member_log(&self) -> Ident {
        Ident::new("log", Span::call_site())
    }

    pub fn mod_transactions_struct_data_member_rollback(&self) -> Ident {
        Ident::new("rollback", Span::call_site())
    }

    pub fn mod_transactions_enum_logitem_variant_update(&self) -> Ident {
        Ident::new("Update", Span::call_site())
    }

    pub fn mod_transactions_enum_logitem_variant_insert(&self) -> Ident {
        Ident::new("Insert", Span::call_site())
    }

    pub fn mod_transactions_enum_logitem_variant_append(&self) -> Ident {
        Ident::new("Append", Span::call_site())
    }

    pub fn mod_transactions_enum_logitem_variant_delete(&self) -> Ident {
        Ident::new("Delete", Span::call_site())
    }

    pub fn mod_delete(&self) -> Ident {
        Ident::new("delete", Span::call_site())
    }

    pub fn trait_delete(&self) -> Ident {
        Ident::new("Delete", Span::call_site())
    }
}
