use crate::{
    operations::{self, SingleOpFn},
    uniques::UniqueDec,
};
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ItemImpl, ItemMod, ItemStruct};

use super::{
    columns::PrimaryKind,
    groups::{Groups, GroupsDef},
    namer::CodeNamer,
    operations::{update::Update, SingleOp},
    predicates::{self, Predicate},
    uniques::{self, Unique},
};

pub struct Table<Primary: PrimaryKind> {
    pub groups: Groups<Primary>,
    pub uniques: Vec<Unique>,
    pub predicates: Vec<Predicate>,
    pub updates: Vec<Update>,
    pub name: Ident,
}

struct TableDec {
    table_struct: Tokens<ItemStruct>,
    table_impl: Tokens<ItemImpl>,
    window_struct: Tokens<ItemStruct>,
}

fn generate_table_and_window(transactions: bool, namer: &CodeNamer) -> TableDec {
    let CodeNamer {
        struct_window,
        struct_table,
        struct_table_member_columns: table_member_columns,
        struct_column_holder,
        struct_window_holder,
        struct_table_member_uniques: table_member_uniques,
        struct_unique,
        mod_transactions,
        mod_transactions_struct_data,
        struct_table_member_transactions: table_member_transactions,
        ..
    } = namer;

    let (trans_table, trans_new, trans_wind, trans_wind_def) = if transactions {
        (
            quote!(#table_member_transactions: #mod_transactions::#mod_transactions_struct_data ),
            quote!(#table_member_transactions: #mod_transactions::#mod_transactions_struct_data::new() ),
            quote!(#table_member_transactions: &mut self.#table_member_transactions),
            quote!(#table_member_transactions: &'imm mut #mod_transactions::#mod_transactions_struct_data),
        )
    } else {
        (quote!(), quote!(), quote!(), quote!())
    };

    TableDec {
        table_struct: quote! {
            pub struct #struct_table {
                #table_member_columns: #struct_column_holder,
                #table_member_uniques: #struct_unique,
                #trans_table
            }
        }
        .into(),
        table_impl: quote! {
            impl #struct_table {
                pub fn new(size_hint: usize) -> Self {
                    Self {
                        #table_member_columns: #struct_column_holder::new(size_hint),
                        #table_member_uniques: #struct_unique::new(size_hint),
                        #trans_new
                    }
                }

                pub fn window(&mut self) -> #struct_window<'_> {
                    #struct_window {
                        #table_member_columns: self.#table_member_columns.window(),
                        #table_member_uniques: &mut self.#table_member_uniques,
                        #trans_wind
                    }
                }
            }
        }
        .into(),
        window_struct: quote! {
            pub struct #struct_window<'imm> {
                #table_member_columns: #struct_window_holder<'imm>,
                #table_member_uniques: &'imm mut #struct_unique,
                #trans_wind_def
            }
        }
        .into(),
    }
}

impl<Primary: PrimaryKind> Table<Primary> {
    pub fn generate(&self, namer: &CodeNamer) -> Tokens<ItemMod> {
        let Self {
            groups,
            uniques,
            predicates,
            updates,
            name,
        } = self;

        let CodeNamer {
            pulpit_path,
            type_key_error,
            ..
        } = namer;

        let column_types = groups.column_types(namer);
        let key_type = groups.key_type(namer);

        let GroupsDef {
            columns_struct,
            columns_impl,
            window_holder_struct,
        } = groups.columns_definition(namer);

        let predicate_mod = predicates::generate(predicates, groups, namer);
        let UniqueDec {
            unique_struct,
            unique_impl,
        } = uniques::generate(uniques, groups, namer);

        let mut ops_mod_code = vec![
            operations::borrow::generate(groups, namer),
            operations::get::generate(groups, namer),
            operations::update::generate(updates, groups, uniques, predicates, namer),
            operations::insert::generate(groups, uniques, predicates, namer),
            operations::unique_get::generate(groups, uniques, namer),
        ];
        if Primary::TRANSACTIONS {
            ops_mod_code.push(operations::transact::generate(groups, updates, namer))
        }

        let mut ops_fn_code = vec![
            operations::count::generate(namer),
            operations::scan::generate(namer),
        ];

        if Primary::DELETIONS {
            ops_fn_code.push(operations::delete::generate(namer, groups, uniques))
        }

        let TableDec {
            table_struct,
            table_impl,
            window_struct,
        } = generate_table_and_window(Primary::TRANSACTIONS, namer);

        let ops_tokens = ops_mod_code
            .into_iter()
            .map(|SingleOp { op_mod, op_impl }| {
                quote! {
                    #op_mod
                    #op_impl
                }
            })
            .chain(
                ops_fn_code
                    .into_iter()
                    .map(|SingleOpFn { op_impl }| quote! { #op_impl }),
            );

        quote! {
            mod #name {
                #![allow(unused, non_camel_case_types)]

                use #pulpit_path::column::{
                    PrimaryWindow,
                    PrimaryWindowApp,
                    PrimaryWindowPull,
                    PrimaryWindowHide,
                    AssocWindow,
                    AssocWindowPull,
                    Column,
                };

                #[derive(Debug)]
                pub struct #type_key_error;

                #column_types

                #(#ops_tokens)*

                #key_type

                #predicate_mod
                #unique_struct
                #unique_impl

                #columns_struct
                #columns_impl
                #window_holder_struct

                #table_struct
                #table_impl
                #window_struct
            }
        }
        .into()
    }
}
