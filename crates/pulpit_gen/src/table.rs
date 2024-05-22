use crate::{operations, uniques::UniqueDec};
use quote::quote;
use quote_debug::Tokens;
use std::collections::{HashMap, HashSet};
use syn::{Ident, ItemImpl, ItemMod, ItemStruct};

use super::{
    columns::PrimaryKind,
    groups::{FieldName, Groups, GroupsDef},
    namer::CodeNamer,
    operations::{update::Update, SingleOp},
    predicates::{self, Predicate},
    uniques::{self, Unique},
};

pub struct Table<Primary: PrimaryKind> {
    pub groups: Groups<Primary>,
    pub uniques: HashMap<FieldName, Unique>,
    pub predicates: Vec<Predicate>,
    pub updates: Vec<Update>,
    pub name: Ident,
}

struct TableDec {
    table_struct: Tokens<ItemStruct>,
    table_impl: Tokens<ItemImpl>,
    window_struct: Tokens<ItemStruct>,
}

pub fn generate_table_and_window(transactions: bool, namer: &CodeNamer) -> TableDec {
    let window_name = namer.struct_window();
    let table_name = namer.struct_table();

    let columns_member = namer.table_member_columns();
    let columns_holder = namer.struct_column_holder();
    let window_holder = namer.struct_window_holder();

    let uniques = namer.table_member_uniques();
    let uniques_type = namer.struct_unique();

    let (trans_table, trans_new, trans_wind, trans_wind_def) = if transactions {
        let transactions_mod = namer.mod_transactions();
        let transaction_type = namer.mod_transactions_struct_data();
        let trans_member = namer.table_member_transactions();
        (
            quote!(#trans_member: #transactions_mod::#transaction_type ),
            quote!(#trans_member: #transactions_mod::#transaction_type::new() ),
            quote!(#trans_member: &mut self.#trans_member),
            quote!(#trans_member: &'imm mut #transactions_mod::#transaction_type),
        )
    } else {
        (quote!(), quote!(), quote!(), quote!())
    };

    TableDec {
        table_struct: quote! {
            pub struct #table_name {
                #columns_member: #columns_holder,
                #uniques: #uniques_type,
                #trans_table
            }
        }
        .into(),
        table_impl: quote! {
            impl #table_name {
                pub fn new(size_hint: usize) -> Self {
                    Self {
                        #columns_member: #columns_holder::new(size_hint),
                        #uniques: #uniques_type::new(size_hint),
                        #trans_new
                    }
                }

                pub fn window(&mut self) -> #window_name<'_> {
                    #window_name {
                        #columns_member: self.#columns_member.window(),
                        #uniques: &mut self.#uniques,
                        #trans_wind
                    }
                }
            }
        }
        .into(),
        window_struct: quote! {
            pub struct #window_name<'imm> {
                #columns_member: #window_holder<'imm>,
                #uniques: &'imm mut #uniques_type,
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
        let pulpit_path = namer.pulpit_path();
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

        let mut ops_code = Vec::new();
        ops_code.push(operations::borrow::generate(groups, namer));
        ops_code.push(operations::get::generate(groups, namer));
        ops_code.push(operations::update::generate(
            updates, groups, uniques, predicates, namer,
        ));
        ops_code.push(operations::insert::generate(groups, namer));
        if Primary::TRANSACTIONS {
            ops_code.push(operations::transact::generate(groups, updates, namer))
        }

        if Primary::DELETIONS {
            ops_code.push(operations::delete::generate(namer))
        }

        let TableDec {
            table_struct,
            table_impl,
            window_struct,
        } = generate_table_and_window(Primary::TRANSACTIONS, namer);

        let ops_tokens = ops_code.into_iter().map(
            |SingleOp {
                 op_mod,
                 op_trait,
                 op_impl,
             }| {
                quote! {
                    #op_mod
                    #op_trait
                    #op_impl
                }
            },
        );

        let key_error_name = namer.type_key_error();

        quote! {
            mod #name {
                use #pulpit_path::column::{
                    PrimaryWindow,
                    PrimaryWindowApp,
                    PrimaryWindowPull,
                    PrimaryWindowHide,
                    AssocWindow,
                    AssocWindowPull,
                    Column,
                };

                pub struct #key_error_name;

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
