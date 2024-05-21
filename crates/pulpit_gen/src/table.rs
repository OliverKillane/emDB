use crate::operations;
use quote::quote;
use quote_debug::Tokens;
use std::collections::{HashMap, HashSet};
use syn::{Ident, ItemMod};

use super::{
    columns::{FieldName, Groups, GroupsDef, PrimaryKind},
    namer::CodeNamer,
    operations::{update::Update, SingleOp},
    predicates::{self, Predicate},
    uniques::{self, Unique},
};

struct Table<Primary: PrimaryKind> {
    groups: Groups<Primary>,
    uniques: HashMap<FieldName, Unique>,
    predicates: Vec<Predicate>,
    updates: Vec<Update>,
    name: Ident,
}

impl<Primary: PrimaryKind> Table<Primary> {
    fn generate(&self, namer: &CodeNamer) -> Tokens<ItemMod> {
        let Self {
            groups,
            uniques,
            predicates,
            updates,
            name,
        } = self;

        let column_types = groups.column_types(namer);
        let key_type = groups.key_type(namer);
        let GroupsDef {
            columns_struct,
            columns_impl,
            window_struct,
        } = groups.columns_definition(namer);

        let predicate_mod = predicates::generate(predicates, groups, namer);
        let unique_struct = uniques::generate(uniques, groups, namer);

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

        quote! {
            mod #name {
                #column_types

                #(#ops_tokens)*

                #key_type

                #predicate_mod
                #unique_struct

                #columns_struct
                #columns_impl
                #window_struct

            }
        }
        .into()
    }
}
