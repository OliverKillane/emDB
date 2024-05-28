use crate::{
    columns::PrimaryKind,
    groups::{FieldIndex, Groups},
    namer::CodeNamer,
    operations::SingleOpFn,
    uniques::Unique,
};
use proc_macro2::Span;
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprMatch, Ident};

pub fn generate<Primary: PrimaryKind>(
    namer: &CodeNamer,
    groups: &Groups<Primary>,
    uniques: &[Unique],
) -> SingleOpFn {
    let CodeNamer {
        type_key_error,
        type_key,
        struct_window,
        name_primary_column,
        struct_table_member_columns: table_member_columns,
        mod_transactions_enum_logitem,
        mod_transactions_enum_logitem_variant_delete,
        mod_transactions,
        struct_table_member_transactions: table_member_transactions,
        mod_transactions_struct_data_member_rollback,
        mod_transactions_struct_data_member_log,
        struct_window_method_delete: method_delete,
        pulpit_path,
        struct_window_method_reverse_insert,
        struct_window_method_delete_hidden,
        struct_window_method_restore_hidden,
        struct_table_member_uniques,
        struct_window_method_borrow,
        ..
    } = namer;
    let key_ident = Ident::new("key", Span::call_site());
    let index_ident = Ident::new("index", Span::call_site());
    let brw_ident = Ident::new("brw_data", Span::call_site());

    assert!(Primary::DELETIONS);

    let unique_deletions = uniques.iter().map(|Unique { alias: _, field }| {
        let field_index = groups.get_field_index(field).unwrap();
        let data = match field_index {
            FieldIndex::Primary(_) => namer.name_primary_column.clone(),
            FieldIndex::Assoc {
                assoc_ind,
                inner: _,
            } => namer.name_assoc_column(*assoc_ind),
        };

        let imm_access = if field_index.is_imm() {
            quote!(imm_data)
        } else {
            quote!(mut_data)
        };
        quote!(self.#struct_table_member_uniques.#field.pull(&#data.#imm_access.#field).unwrap())
    });

    let assoc_cols = (0..groups.assoc.len())
        .map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(let #name = unsafe { self.#table_member_columns.#name.pull(#index_ident) })
        })
        .collect::<Vec<_>>();

    let delete_hard: Tokens<ExprMatch> = quote!{
            match self.#table_member_columns.#name_primary_column.pull(#key_ident) {
                Ok(#pulpit_path::column::Entry{ index: #index_ident, data: #name_primary_column }) => {
                    #(#assoc_cols;)*
                    #(#unique_deletions;)*
                    Ok(())
                },
                Err(_) => Err(#type_key_error),
            }
    }.into();

    let op_impl = if Primary::TRANSACTIONS {
        let transactional = quote! {
            if !self.#table_member_transactions.#mod_transactions_struct_data_member_rollback {
                self.#table_member_transactions.#mod_transactions_struct_data_member_log.push(#mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_delete(key));
            }
        };
        let restore_unique_from_borrow = uniques.iter().map(|Unique { alias:_, field }| {
            quote!(self.#struct_table_member_uniques.#field.insert(#brw_ident.#field.clone(), #key_ident).unwrap())
        });

        quote! {
            impl <'imm> #struct_window<'imm> {
                fn #struct_window_method_reverse_insert(&mut self, #key_ident: #type_key) {
                    debug_assert!(self.#table_member_transactions.#mod_transactions_struct_data_member_rollback);
                    {
                        #delete_hard
                    }.unwrap()
                }

                fn #struct_window_method_delete_hidden(&mut self, #key_ident: #type_key) {
                    debug_assert!(self.#table_member_transactions.#mod_transactions_struct_data_member_rollback);
                    let #pulpit_path::column::Entry{ index: #index_ident, data } = self.#table_member_columns.#name_primary_column.pull(key).unwrap();
                    unsafe {
                        #(#assoc_cols;)*
                    }
                }

                fn #struct_window_method_restore_hidden(&mut self, #key_ident: #type_key) {
                    debug_assert!(self.#table_member_transactions.#mod_transactions_struct_data_member_rollback);
                    self.#table_member_columns.#name_primary_column.reveal(#key_ident).unwrap();
                    let #brw_ident = self.#struct_window_method_borrow(#key_ident).unwrap();
                    #(#restore_unique_from_borrow;)*
                }

                pub fn #method_delete(&mut self, #key_ident: #type_key) -> Result<(), #type_key_error> {
                    match self.#table_member_columns.#name_primary_column.hide(#key_ident) {
                        Ok(()) => (),
                        Err(_) => return Err(#type_key_error),
                    }
                    #transactional
                    Ok(())
                }
            }
        }
        .into()
    } else {
        quote!{
            impl <'imm> #struct_window<'imm> {
                pub fn #method_delete(&mut self, #key_ident: #type_key) -> Result<(), #type_key_error> {
                    #delete_hard
                }
            }
        }.into()
    };

    SingleOpFn { op_impl }
}
