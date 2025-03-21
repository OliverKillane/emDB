use super::namer::SerializedNamer;
use crate::{
    backend::interface::{namer::InterfaceNamer, public::exposed_keys, InterfaceTrait},
    plan,
};
use pulpit::gen::selector::{SelectorImpl, TableSelectors};
use quote::{quote, ToTokens};
use quote_debug::Tokens;
use std::collections::HashMap;
use syn::{Ident, ItemImpl, ItemMod, ItemStruct, Type};

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
pub fn generate_tables<'imm>(
    lp: &'imm plan::Plan,
    interface_trait: &Option<InterfaceTrait>,
    namer: &SerializedNamer,
    selector: &TableSelectors,
    inlining: bool,
) -> TableWindow<'imm> {
    // get the constraints and fields of each table
    let mut pulpit_configs = lp
        .tables
        .iter()
        .map(|(key, emdb_table)| {
            let pulpit_select = pulpit::gen::selector::SelectOperations {
                name: namer.table_internal_name(lp, key),
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
                    if let Some(plan::Constraint {
                        alias,
                        cons: plan::Limit(expr),
                    }) = &emdb_table.row_cons.limit
                    {
                        Some(pulpit::gen::limit::Limit {
                            value: pulpit::gen::limit::LimitKind::ConstVal(
                                expr.into_token_stream().into(),
                            ),
                            alias: alias.clone(),
                        })
                    } else {
                        None
                    }
                },
                updates: Vec::new(),
                gets: Vec::new(),
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
            plan::Operator::DeRef(plan::DeRef {
                table, named_type, ..
            }) => {
                let deref_fields = lp
                    .get_record_type_conc(*named_type)
                    .fields
                    .keys()
                    .map(|rf| namer.transform_field_name(rf))
                    .collect::<Vec<_>>();
                pulpit_configs
                    .get_mut(&plan::Idx::new(*table, lp))
                    .unwrap()
                    .gets
                    .push(pulpit::gen::operations::get::Get {
                        fields: deref_fields,
                        alias: namer.pulpit_table_interaction(key),
                    });
            }
            _ => (),
        }
    }

    let SerializedNamer {
        pulpit:
            ref pulpit_namer @ pulpit::gen::namer::CodeNamer {
                struct_table,
                struct_window,
                type_key,
                ..
            },
        struct_datastore,
        struct_database,
        db_lifetime,
        mod_tables,
        struct_stats,
        struct_datastore_member_stats,
        struct_database_member_stats,
        ..
    } = namer;

    let (get_types, gen_data): (HashMap<_, _>, Vec<_>) = pulpit_configs
        .into_iter()
        .map(|(key, config)| {
            let table_impl = selector.select_table(config);
            (
                (key, table_impl.op_get_types(pulpit_namer)),
                (
                    (key, table_impl.insert_can_error()),
                    table_impl.generate(
                        pulpit_namer,
                        if inlining {
                            vec![pulpit::gen::table::AttrKinds::Inline]
                        } else {
                            vec![]
                        },
                    ),
                ),
            )
        })
        .unzip();

    let (insert_can_error, table_defs): (HashMap<_, _>, Vec<_>) = gen_data.into_iter().unzip();

    let table_mod_names = lp
        .tables
        .iter()
        .map(|(k, _)| namer.table_internal_name(lp, k))
        .collect::<Vec<_>>();

    let datastore_members = table_mod_names
        .iter()
        .map(|mod_name| quote!(#mod_name: #mod_tables::#mod_name::#struct_table));
    let datastore_members_new = table_mod_names
        .iter()
        .map(|mod_name| quote!(#mod_name: #mod_tables::#mod_name::#struct_table::new(1024)));

    let (database_members_window_stream, database_members_stream): (Vec<_>, Vec<_>) = table_mod_names
        .iter()
        .map(|mod_name| {
            (
                quote!(#mod_name: self.#mod_name.window()),
                quote!(#mod_name: #mod_tables::#mod_name::#struct_window<#db_lifetime>),
            )
        })
        .unzip();

    let (database_members_window, database_members) = if database_members_stream.is_empty() {
        (
            quote!(phantom: std::marker::PhantomData,),
            quote!(phantom: std::marker::PhantomData<&#db_lifetime ()>,),
        )
    } else {
        (
            quote!(#(#database_members_window_stream,)*),
            quote!(#(#database_members_stream,)*),
        )
    };

    let InterfaceNamer {
        trait_datastore,
        trait_datastore_method_db,
        trait_datastore_method_new,
        trait_datastore_type_database,
        ..
    } = &namer.interface;

    let (impl_datastore, modifiers, key_defs, ds_assoc_db) = if let Some(InterfaceTrait { name }) =
        interface_trait
    {
        let exposed_table_keys = exposed_keys(lp);
        (
            quote! { super::#name::#trait_datastore for },
            quote!(),
            exposed_table_keys
                .into_iter()
                .map(|tablekey| {
                    let mod_name = namer.table_internal_name(lp, *tablekey);
                    let table_name = &lp.get_table(*tablekey).name;
                    let key_name = namer.interface.key_name(table_name);
                    quote! { type #key_name = #mod_tables::#mod_name::#type_key }
                })
                .collect::<Vec<_>>(),
            quote!(type #trait_datastore_type_database<#db_lifetime> = #struct_database<#db_lifetime>;),
        )
    } else {
        (quote!(), quote!(pub), Vec::new(), quote!())
    };

    TableWindow {
        table_defs,
        datastore: quote! {
            pub struct #struct_datastore {
                #(#datastore_members,)*
                #struct_datastore_member_stats: #struct_stats,
            }
        }
        .into(),
        datastore_impl: quote! {
            impl #impl_datastore #struct_datastore {
                #ds_assoc_db
                #(#key_defs;)*

                #modifiers fn #trait_datastore_method_new() -> Self {
                    Self {
                        #(#datastore_members_new,)*
                        #struct_datastore_member_stats: #struct_stats::default(),
                    }
                }

                #modifiers fn #trait_datastore_method_db(&mut self) -> #struct_database<'_> {
                    #struct_database {
                        #database_members_window
                        #struct_database_member_stats: &self.#struct_datastore_member_stats,        
                    }
                }
            }
        }
        .into(),
        database: quote! {
            pub struct #struct_database<#db_lifetime> {
                #database_members
                #struct_database_member_stats: &#db_lifetime #struct_stats,
            }
        }
        .into(),
        table_generated_info: GeneratedInfo {
            get_types,
            insert_can_error,
        },
    }
}
