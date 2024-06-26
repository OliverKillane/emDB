use crate::{
    groups::{FieldIndex, Groups},
    namer::CodeNamer,
    operations::SingleOpFn,
    uniques::Unique,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprMatch, Ident};

/// Should only be called when deletions are enabled for the table
pub fn generate(
    namer: &CodeNamer,
    groups: &Groups,
    uniques: &[Unique],
    transactions: bool,
    op_attrs: &TokenStream,
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

        // POSSIBLE BUG: pulling values does not consider the transformations
        //               that may need to be applied to immutable values
        //               (`ImmPull`), or autodereference might take care of
        //               this - not failing any tests for retain, would fail for
        //               other wrappings?
        quote!(self.#struct_table_member_uniques.#field.pull(&#data.#imm_access.#field).unwrap())
    });

    let assoc_cols = (0..groups.assoc.len())
        .map(|ind| {
            let name = namer.name_assoc_column(ind);
            quote!(let #name = unsafe { self.#table_member_columns.#name.assoc_pull(#index_ident) })
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

    let op_impl = if transactions {
        let transactional = quote! {
            if !self.#table_member_transactions.#mod_transactions_struct_data_member_rollback {
                self.#table_member_transactions.#mod_transactions_struct_data_member_log.push(#mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_delete(key));
            }
        };
        // We cannot insert into the table while holding the borrow of the row.
        // - while borrow does not impact the unique indices, the `self.borrow`
        //   borrows all members of `self`
        // - Hence we clone, then place, rather than just `self.uniques.insert(brw.field.clone())`
        let get_clone_of_uniques = uniques
            .iter()
            .map(|Unique { alias, field }| quote!(let #alias = #brw_ident.#field.clone()));
        let restore_unique_from_borrow = uniques.iter().map(|Unique { alias, field }| {
            quote!(self.#struct_table_member_uniques.#field.insert(#alias, #key_ident).unwrap())
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
                    debug_assert!(!self.#table_member_transactions.#mod_transactions_struct_data_member_rollback);
                    let #pulpit_path::column::Entry{ index: #index_ident, data } = self.#table_member_columns.#name_primary_column.pull(key).unwrap();
                    unsafe {
                        #(#assoc_cols;)*
                    }
                }

                fn #struct_window_method_restore_hidden(&mut self, #key_ident: #type_key) {
                    debug_assert!(self.#table_member_transactions.#mod_transactions_struct_data_member_rollback);
                    self.#table_member_columns.#name_primary_column.reveal(#key_ident).unwrap();
                    let #brw_ident = self.#struct_window_method_borrow(#key_ident).unwrap();
                    #(#get_clone_of_uniques;)*
                    #(#restore_unique_from_borrow;)*
                }

                #op_attrs
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
                #op_attrs
                pub fn #method_delete(&mut self, #key_ident: #type_key) -> Result<(), #type_key_error> {
                    #delete_hard
                }
            }
        }.into()
    };

    SingleOpFn { op_impl }
}
