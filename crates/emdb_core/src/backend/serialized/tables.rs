use quote::{quote, ToTokens};
use quote_debug::Tokens;
use std::collections::HashMap;
use syn::{Ident, ItemImpl, ItemMod, ItemStruct, Type};

use super::namer::SerializedNamer;
use crate::plan;

pub struct GeneratedInfo<'imm> {
    pub get_types: HashMap<plan::Idx<'imm, plan::Table>, HashMap<Ident, Tokens<Type>>>,
    pub insert_can_error: HashMap<plan::Idx<'imm, plan::Table>, bool>, // TODO: hashset instead?
}

pub struct TableWindow<'imm> {
    pub table_defs: Vec<Tokens<ItemMod>>,
    pub datastore: Tokens<ItemStruct>,
    pub datastore_impl: Tokens<ItemImpl>,
    pub database: Tokens<ItemStruct>,
    pub table_generated_info: GeneratedInfo<'imm>,
}

/// Generate the tokens for the tables, and the struct to hold them (in [`TableWindow`]).
/// - Generates the tokens for the [`plan::ScalarType`]s of table fields assuming they are just [`plan::ScalarTypeConc::Rust`] tyes
pub fn generate_tables<'imm>(lp: &'imm plan::Plan, namer: &SerializedNamer) -> TableWindow<'imm> {
    // get the constraints and fields of each table
    let mut pulpit_configs = lp
        .tables
        .iter()
        .map(|(key, emdb_table)| {
            let pulpit_select = pulpit::gen::selector::SelectOperations {
                name: emdb_table.name.clone(),
                transactions: true,
                deletions: false,
                fields: emdb_table
                    .columns
                    .iter()
                    .map(|(field, v)| {
                        (
                            namer.transform_field_name(field),
                            match lp.get_scalar_type_conc(v.data_type) {
                                plan::ScalarTypeConc::Rust {
                                    type_context: plan::TypeContext::DataStore,
                                    ty,
                                } => ty.to_token_stream().into(),
                                _ => unreachable!("Only Rust types are allowed in the data store"),
                            },
                        )
                    })
                    .collect(),
                uniques: emdb_table
                    .columns
                    .iter()
                    .filter_map(|(field, v)| {
                        v.cons
                            .unique
                            .as_ref()
                            .map(|a| pulpit::gen::uniques::Unique {
                                alias: a.alias.clone(),
                                field: namer.transform_field_name(field),
                            })
                    })
                    .collect(),
                predicates: emdb_table
                    .row_cons
                    .preds
                    .iter()
                    .map(|pred| pulpit::gen::predicates::Predicate {
                        alias: pred.alias.clone(),
                        tokens: pred.cons.0.to_token_stream().into(),
                    })
                    .collect(),
                limit: {
                    if let Some(plan::Constraint { alias, cons: plan::Limit (expr)}) = &emdb_table.row_cons.limit {
                        Some(pulpit::gen::limit::Limit { value: pulpit::gen::limit::LimitKind::ConstVal(expr.into_token_stream().into()), alias: alias.clone() })
                    } else {
                        None
                    }
                },
                updates: Vec::new(),
                public: true,
            };

            (plan::Idx::new(key, lp), pulpit_select)
        })
        .collect::<HashMap<_, _>>();

    // get the updates and deletions
    for (key, op) in &lp.operators {
        match op {
            plan::Operator::Update(plan::Update { table, mapping, .. }) => pulpit_configs
                .get_mut(&plan::Idx::new(*table, lp))
                .unwrap()
                .updates
                .push(pulpit::gen::operations::update::Update {
                    fields: mapping
                        .keys()
                        .map(|rec| namer.transform_field_name(rec))
                        .collect(),
                    alias: namer.pulpit_table_interaction(key),
                }),
            plan::Operator::Delete(plan::Delete { table, .. }) => {
                pulpit_configs
                    .get_mut(&plan::Idx::new(*table, lp))
                    .unwrap()
                    .deletions = true;
            }
            _ => (),
        }
    }

    let SerializedNamer {
        pulpit:
            ref pulpit_namer @ pulpit::gen::namer::CodeNamer {
                struct_table,
                struct_window,
                ..
            },
        struct_datastore,
        struct_database,
        db_lifetime,
        mod_tables,
        ..
    } = namer;

    let (get_types, gen_data): (HashMap<_, _>, Vec<_>) = pulpit_configs
        .into_iter()
        .map(|(key, config)| {
            let table_impl = pulpit::gen::selector::basic::selector(config);
            (
                (key, table_impl.op_get_types(pulpit_namer)),
                (
                    (key, table_impl.insert_can_error()),
                    table_impl.generate(pulpit_namer),
                ),
            )
        })
        .unzip();

    let (insert_can_error, table_defs): (HashMap<_, _>, Vec<_>) = gen_data.into_iter().unzip();

    let table_names = lp
        .tables
        .iter()
        .map(|(_, table)| &table.name)
        .collect::<Vec<_>>();

    let datastore_members = table_names
        .iter()
        .map(|name| quote!(#name: #mod_tables::#name::#struct_table));
    let datastore_members_new = table_names
        .iter()
        .map(|name| quote!(#name: #mod_tables::#name::#struct_table::new(1024)));

    let (database_members_window_stream, database_members_stream): (Vec<_>, Vec<_>) = table_names
        .iter()
        .map(|name| (quote!(#name: self.#name.window()), quote!(#name: #mod_tables::#name::#struct_window<#db_lifetime>))).unzip();

    let (database_members_window, database_members) = if database_members_stream.is_empty() {
        (quote!(phantom: std::marker::PhantomData), quote!(phantom: std::marker::PhantomData<&#db_lifetime ()>))
    } else {
        (quote!(#(#database_members_window_stream,)*),quote!(#(#database_members_stream,)*))
    };

    TableWindow {
        table_defs,
        datastore: quote! {
            pub struct #struct_datastore {
                #(#datastore_members,)*
            }
        }
        .into(),
        datastore_impl: quote! {
            impl #struct_datastore {
                pub fn new() -> Self {
                    Self {
                        #(#datastore_members_new,)*
                    }
                }

                pub fn db(&mut self) -> #struct_database<'_> {
                    #struct_database {
                        #database_members_window
                    }
                }
            }
        }
        .into(),
        database: quote! {
            pub struct #struct_database<#db_lifetime> {
                #database_members
            }
        }
        .into(),
        table_generated_info: GeneratedInfo {
            get_types,
            insert_can_error,
        },
    }
}
