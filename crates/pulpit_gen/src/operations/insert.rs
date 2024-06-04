use std::iter::once;

use super::SingleOp;
use crate::{
    columns::ColKind,
    groups::{Field, Group, Groups},
    namer::CodeNamer,
    predicates::Predicate,
    uniques::Unique,
};
use proc_macro2::Span;
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprLet, Ident};

pub fn generate_column_assignments<Col: ColKind>(
    name: &Ident,
    insert_val: &Ident,
    group: &Group<Col>,
    namer: &CodeNamer,
) -> Tokens<ExprLet> {
    let imm_data_fields = group.fields.imm_fields.iter().map(|Field { name, .. }| {
        quote! {#name: #insert_val.#name}
    });
    let mut_data_fields = group.fields.mut_fields.iter().map(|Field { name, .. }| {
        quote! {#name: #insert_val.#name}
    });

    let CodeNamer {
        mod_columns,
        mod_columns_struct_imm,
        mod_columns_struct_mut,
        pulpit_path,
        ..
    } = namer;

    // TODO: remove extra brackets, as ExprLet parsing of struct literals has
    //       been fixed in syn (see https://github.com/dtolnay/syn/issues/1670)
    quote! {
        let #name = (#pulpit_path::column::Data {
            imm_data: #mod_columns::#name::#mod_columns_struct_imm {
                #(#imm_data_fields,)*
            },
            mut_data: #mod_columns::#name::#mod_columns_struct_mut {
                #(#mut_data_fields,)*
            }
        })
    }
    .into()
}

pub fn generate(
    groups: &Groups,
    uniques: &[Unique],
    predicates: &[Predicate],
    namer: &CodeNamer,
    deletions: bool,
    transactions: bool,
) -> SingleOp {
    let CodeNamer {
        type_key,
        struct_window,
        mod_insert,
        mod_insert_struct_insert,
        mod_insert_enum_error,
        mod_borrow,
        mod_borrow_struct_borrow,
        mod_predicates,
        struct_table_member_uniques: table_member_uniques,
        struct_table_member_columns: table_member_columns,
        pulpit_path,
        name_primary_column,
        mod_transactions_enum_logitem,
        mod_transactions_enum_logitem_variant_insert,
        mod_transactions_enum_logitem_variant_append,
        mod_transactions,
        struct_table_member_transactions: table_member_transactions,
        mod_transactions_struct_data_member_rollback,
        mod_transactions_struct_data_member_log,
        struct_window_method_insert: method_insert,
        ..
    } = namer;

    let insert_val = Ident::new("insert_val", Span::call_site());
    let key_var = Ident::new("key", Span::call_site());

    let insert_struct_fields = groups.idents.iter().map(|(field_name, field_index)| {
        let ty = groups.get_type(field_index);
        quote!(pub #field_name: #ty)
    });

    let predicate_args = groups
        .idents
        .keys()
        .map(|k| quote! {#k : &#insert_val.#k})
        .collect::<Vec<_>>();

    let predicate_checks = predicates.iter().map(|Predicate { alias, tokens: _ }| {
        quote! {
            if !#mod_predicates::#alias(#mod_borrow::#mod_borrow_struct_borrow{#(#predicate_args),*}) {
                return Err(#mod_insert::#mod_insert_enum_error::#alias);
            }
        }
    });

    let errors = uniques.iter().map(|Unique { alias, .. }| alias).chain(
        predicates
            .iter()
            .map(|Predicate { alias, tokens: _ }| alias),
    );

    let unique_checks = uniques.iter().map(|Unique { alias, field }| {
        quote! {
            let #alias = match self.#table_member_uniques.#field.lookup(&#insert_val.#field) {
                Ok(_) => return Err(#mod_insert::#mod_insert_enum_error::#alias),
                Err(_) => #insert_val.#field.clone(),
            };
        }
    });

    let unique_updates = uniques.iter().map(|Unique { alias, field }| {
        quote! {
            self.#table_member_uniques.#field.insert(#alias, #key_var).unwrap();
        }
    });

    let splitting = once(generate_column_assignments(
        &namer.name_primary_column.clone(),
        &insert_val,
        &groups.primary,
        namer,
    ))
    .chain((0..groups.assoc.len()).map(|ind| {
        generate_column_assignments(
            &namer.name_assoc_column(ind),
            &insert_val,
            &groups.assoc[ind],
            namer,
        )
    }));

    let assoc_grps = (0..groups.assoc.len()).map(|ind| namer.name_assoc_column(ind));
    let appends = assoc_grps.clone().map(|grp| {
        quote! {
            self.#table_member_columns.#grp.append(#grp);
        }
    });

    let (add_action, add_trans) = if deletions {
        let places = assoc_grps.map(|grp| {
            quote! {
                self.#table_member_columns.#grp.place(index, #grp);
            }
        });
        (
            quote! {
                let (#key_var, action) = self.#table_member_columns.#name_primary_column.insert(#name_primary_column);
                match action {
                    #pulpit_path::column::InsertAction::Place(index) => {
                        unsafe {
                            #(#places)*
                        }
                    },
                    #pulpit_path::column::InsertAction::Append => {
                        #(#appends)*
                    }
                }
            },
            if transactions {
                quote! {
                    if !self.#table_member_transactions.#mod_transactions_struct_data_member_rollback {
                        self.#table_member_transactions.#mod_transactions_struct_data_member_log.push(#mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_insert(key));
                    }
                }
            } else {
                quote!()
            },
        )
    } else {
        (
            quote! {
                let #key_var = self.#table_member_columns.#name_primary_column.append(#name_primary_column);
                #(#appends)*
            },
            if transactions {
                quote! {
                    if !self.#table_member_transactions.#mod_transactions_struct_data_member_rollback {
                        self.#table_member_transactions.#mod_transactions_struct_data_member_log.push(#mod_transactions::#mod_transactions_enum_logitem::#mod_transactions_enum_logitem_variant_append);
                    }
                }
            } else {
                quote!()
            },
        )
    };

    if uniques.is_empty() && predicates.is_empty() {
        SingleOp {
            op_mod: quote! {
                pub mod #mod_insert {
                    pub struct #mod_insert_struct_insert {
                        #(#insert_struct_fields,)*
                    }
                }
            }
            .into(),
            op_impl: quote! {
                impl <'imm> #struct_window<'imm> {
                    pub fn #method_insert(&mut self, #insert_val: #mod_insert::#mod_insert_struct_insert) -> #type_key {
                        #(#splitting;)*
                        #add_action
                        #add_trans
                        key
                    }
                }
            }
            .into(),
        }
    } else {
        SingleOp {
            op_mod: quote! {
                pub mod #mod_insert {
                    pub struct #mod_insert_struct_insert {
                        #(#insert_struct_fields,)*
                    }
                    #[derive(Debug)]
                    pub enum #mod_insert_enum_error {
                        #(#errors,)*
                    }
                }
            }
            .into(),
            op_impl: quote! {
                impl <'imm> #struct_window<'imm> {
                    pub fn #method_insert(&mut self, #insert_val: #mod_insert::#mod_insert_struct_insert) -> Result<#type_key, #mod_insert::#mod_insert_enum_error> {
                        #(#predicate_checks)*
                        #(#unique_checks)*
                        #(#splitting;)*
                        #add_action
                        #(#unique_updates)*
                        #add_trans

                        Ok(#key_var)
                    }
                }
            }
            .into(),
        }
    }
}
