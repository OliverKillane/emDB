use std::{collections::HashMap, iter::once};

use super::SingleOp;
use crate::{
    columns::{ColKind, PrimaryKind},
    groups::{Field, FieldName, Group, Groups},
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

    let mod_columns = namer.mod_columns();
    let mod_columns_struct_imm = namer.mod_columns_struct_imm();
    let mod_columns_struct_mut = namer.mod_columns_struct_mut();

    let pulpit_path = namer.pulpit_path();
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

pub fn generate<Primary: PrimaryKind>(
    groups: &Groups<Primary>,
    uniques: &HashMap<FieldName, Unique>,
    predicates: &[Predicate],
    namer: &CodeNamer,
) -> SingleOp {
    let type_key = namer.type_key();
    let struct_window = namer.struct_window();
    let mod_insert = namer.mod_insert();
    let mod_insert_struct_insert = namer.mod_insert_struct_insert();
    let mod_insert_enum_error = namer.mod_insert_enum_error();
    let trait_insert = namer.trait_insert();
    let borrow_mod = namer.mod_borrow();
    let mod_borrow_struct_borrow = namer.mod_borrow_struct_borrow();
    let mod_predicates = namer.mod_predicates();
    let table_member_uniques = namer.table_member_uniques();
    let table_member_columns = namer.table_member_columns();
    let pulpit_path = namer.pulpit_path();
    let name_primary_column = namer.name_primary_column();
    let mod_transactions_enum_logitem = namer.mod_transactions_enum_logitem();
    let mod_transactions_enum_logitem_variant_insert =
        namer.mod_transactions_enum_logitem_variant_insert();
    let mod_transactions_enum_logitem_variant_append =
        namer.mod_transactions_enum_logitem_variant_append();
    let mod_transactions = namer.mod_transactions();
    let table_member_transactions = namer.table_member_transactions();
    let mod_transactions_struct_data_member_rollback =
        namer.mod_transactions_struct_data_member_rollback();
    let mod_transactions_struct_data_member_log = namer.mod_transactions_struct_data_member_log();

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

    let predicate_checks = predicates.iter().map(|Predicate { alias, tokens }| {
        quote! {
            if !#mod_predicates::#alias(#borrow_mod::#mod_borrow_struct_borrow{#(#predicate_args),*}) {
                return Err(#mod_insert::#mod_insert_enum_error::#alias);
            }
        }
    });

    let errors = uniques
        .iter()
        .map(|(_, Unique { alias })| alias)
        .chain(predicates.iter().map(|Predicate { alias, tokens }| alias));

    let unique_checks = uniques.iter().map(|(field, Unique { alias })| {
        quote! {
            let #alias = match self.#table_member_uniques.#field.lookup(&#insert_val.#field) {
                Ok(_) => return Err(#mod_insert::#mod_insert_enum_error::#alias),
                Err(_) => #insert_val.#field.clone(),
            };
        }
    });

    let unique_updates = uniques.iter().map(|(field, Unique { alias })| {
        quote! {
            self.#table_member_uniques.#field.insert(#alias, #key_var).unwrap();
        }
    });

    let splitting = once(generate_column_assignments(
        &namer.name_primary_column(),
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

    let (add_action, add_trans) = if Primary::DELETIONS {
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
            if Primary::TRANSACTIONS {
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
                self.#table_member_columns.#name_primary_column.append(#name_primary_column);
                #(#appends)*
            },
            if Primary::TRANSACTIONS {
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
                    /// TODO
                    pub struct #mod_insert_struct_insert {
                        #(#insert_struct_fields,)*
                    }
                }
            }
            .into(),
            op_trait: quote! {
                pub trait #trait_insert {
                    fn insert(&mut self, #insert_val: #mod_insert::#mod_insert_struct_insert) -> #type_key;
                }
            }
            .into(),
            op_impl: quote! {
                impl <'imm> #trait_insert for #struct_window<'imm> {
                    fn insert(&mut self, #insert_val: #mod_insert::#mod_insert_struct_insert) -> #type_key {
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
                    /// TODO
                    pub struct #mod_insert_struct_insert {
                        #(#insert_struct_fields,)*
                    }
                    /// TODO
                    #[derive(Debug)]
                    pub enum #mod_insert_enum_error {
                        #(#errors,)*
                    }
                }
            }
            .into(),
            op_trait: quote! {
                pub trait #trait_insert {
                    fn insert(&mut self, #insert_val: #mod_insert::#mod_insert_struct_insert) -> Result<#type_key, #mod_insert::#mod_insert_enum_error>;
                }
            }
            .into(),
            op_impl: quote! {
                impl <'imm> #trait_insert for #struct_window<'imm> {
                    fn insert(&mut self, #insert_val: #mod_insert::#mod_insert_struct_insert) -> Result<#type_key, #mod_insert::#mod_insert_enum_error> {
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
