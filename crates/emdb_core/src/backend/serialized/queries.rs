use std::collections::{HashMap, HashSet};

use pulpit::gen::namer::CodeNamer;
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprBlock, Ident, ImplItemFn, ItemEnum, ItemImpl, ItemMod, Path};

use crate::{
    backend::interface::{namer::InterfaceNamer, InterfaceTrait}, plan, utils::misc::{PushMap, PushSet}
};

use super::{
    closures::{generate_application, ContextGen},
    namer::SerializedNamer,
    tables::GeneratedInfo,
    types::generate_scalar_type,
};

fn generate_errors(
    errors: HashMap<Ident, Option<Tokens<Path>>>,
    SerializedNamer {
        mod_queries_mod_query_enum_error,
        ..
    }: &SerializedNamer,
) -> Option<Tokens<ItemEnum>> {
    if errors.is_empty() {
        None
    } else {
        let variants = errors.iter().map(|(name, inner)| {
            if let Some(path) = inner {
                quote!(#name(#path))
            } else {
                quote!(#name)
            }
        });
        Some(
            quote! {
                #[derive(Debug)]
                pub enum #mod_queries_mod_query_enum_error {
                    #(#variants),*
                }
            }
            .into(),
        )
    }
}

struct CommitInfo {
    commits: Tokens<ExprBlock>,
    aborts: Tokens<ExprBlock>,
}

fn generate_commits<'imm>(
    lp: &'imm plan::Plan,
    mutated_tables: HashSet<plan::ImmKey<'imm, plan::Table>>,
    SerializedNamer {
        pulpit:
            CodeNamer {
                struct_window_method_commit,
                struct_window_method_abort,
                ..
            },
        ..
    }: &SerializedNamer,
) -> Option<CommitInfo> {
    if mutated_tables.is_empty() {
        None
    } else {
        let (commits, aborts): (Vec<_>, Vec<_>) = mutated_tables
            .iter()
            .map(|key| {
                let table_name = &lp.get_table(**key).name;
                (
                    quote! {
                        self.#table_name.#struct_window_method_commit();
                    },
                    quote! {
                        self.#table_name.#struct_window_method_abort();
                    },
                )
            })
            .unzip();
        Some(CommitInfo {
            commits: quote! { { #(#commits;)* } }.into(),
            aborts: quote! { { #(#aborts;)*  } }.into(),
        })
    }
}

struct QueryMod {
    query_mod: Tokens<ItemMod>,
    query_impl: Tokens<ImplItemFn>,
}

impl QueryMod {
    fn extract(self) -> (Tokens<ItemMod>, Tokens<ImplItemFn>) {
        (self.query_mod, self.query_impl)
    }
}

fn generate_query<'imm>(
    lp: &'imm plan::Plan,
    gen_info: &GeneratedInfo<'imm>,
    namer: &SerializedNamer,
    plan::Query { name, ctx }: &'imm plan::Query,
) -> QueryMod {
    let SerializedNamer {
        qy_lifetime,
        mod_queries,
        mod_queries_mod_query_enum_error,
        method_query_operator_alias,
        ..
    } = namer;

    let context = lp.get_context(*ctx);
    let return_type = if let Some(ret) = context.get_return_type(lp) {
        let ty = namer.record_name(ret);
        quote!(#ty)
    } else {
        quote!(())
    };

    let (params_use, params): (Vec<_>, Vec<_>) = context.params.iter().map(|(name, ty_key)| {
        let ty = generate_scalar_type(lp, &gen_info.get_types, *ty_key, namer);
        (name, quote!(#name: #ty))
    }).unzip();

    let mut errors = HashMap::new();
    let mut mutated_tables = HashSet::new();

    let ContextGen { code, .. } = generate_application(
        lp,
        *ctx,
        &quote!(#mod_queries::#name::#mod_queries_mod_query_enum_error).into(),
        &mut PushMap::new(&mut errors),
        &mut PushSet::new(&mut mutated_tables),
        gen_info,
        namer,
    );

    let run_query = quote!((#code)(self, #(#params_use),* ));

    match (
        generate_errors(errors, namer),
        generate_commits(lp, mutated_tables, namer)
    ) {
        (None, None) => {
            QueryMod {
                query_mod: quote! { mod #name {} }.into(),
                query_impl: quote! {
                    fn #name<#qy_lifetime>(&#qy_lifetime self, #(#params),* ) -> #return_type {
                        #run_query
                    }
                }
                .into(),
            }
        },
        (None, Some(CommitInfo { commits, aborts:_ } )) => {

            // NOTE: This case is possible when many inserts (that do not throw errors) occur on a table, 
            //       but nothing else does. In this case we are not optimal - we could avoid transactions
            //       entirely.
            // TODO: Consider this case (e.g. should we add in alloc errors -> in which case there are no error free inserts?) 

            QueryMod {
                query_mod: quote! { mod #name {} }.into(),
                query_impl: quote! {
                    fn #name<#qy_lifetime>(&#qy_lifetime mut self, #(#params),* ) -> #return_type {
                        let result = #run_query;
                        #commits
                        result
                    }
                }
                .into(),
            }
         },
        (Some(error_enum), None) => {
            QueryMod {
                query_mod: quote!{ pub mod #name {
                    #error_enum
                } }.into(),
                query_impl: quote!{
                    fn #name<#qy_lifetime>(&#qy_lifetime self, #(#params),* ) -> Result<#return_type, #mod_queries::#name::#mod_queries_mod_query_enum_error> {
                        #run_query.map(#method_query_operator_alias::export_single)
                    }
                }.into(),
            }
        }
        (Some(error_enum), Some(CommitInfo { commits, aborts })) => {
            QueryMod {
                query_mod: quote!{ pub mod #name {
                    #error_enum
                } }.into(),
                query_impl: quote!{
                    fn #name<#qy_lifetime>(&#qy_lifetime mut self, #(#params),* ) -> Result<#return_type, #mod_queries::#name::#mod_queries_mod_query_enum_error> {
                        match #run_query {
                            Ok(result) => {
                                #commits
                                Ok(#method_query_operator_alias::export_single(result))
                            },
                            Err(e) => {
                                #aborts
                                Err(e)
                            }
                        }
                    }
                }.into(),
            }
        }
    }
}

pub struct QueriesInfo {
    pub query_mod: Tokens<ItemMod>,

    /// If there are no queries, we should not produce an impl block that does not 
    /// use the [`SimpleNamer::db_lifetime`] as this will cause an error with span
    /// [`proc_macro2::Span::call_site`]
    pub query_impls: Option<Tokens<ItemImpl>>,
}

// TODO: determine error type
// get if an insert for a table has errors, if so, do thingy

pub fn generate_queries<'imm>(
    lp: &'imm plan::Plan,
    gen_info: &GeneratedInfo<'imm>,
    interface_trait: &Option<InterfaceTrait>,
    namer: &'imm SerializedNamer,
) -> QueriesInfo {
    let SerializedNamer {
        db_lifetime,
        mod_queries,
        struct_database,
        struct_datastore,
        interface: InterfaceNamer { trait_database, trait_database_type_datastore, ..},
        ..
    } = namer;
    let (mods, impls): (Vec<Tokens<ItemMod>>, Vec<Tokens<ImplItemFn>>) = lp
        .queries
        .iter()
        .map(move |(_, query)| generate_query(lp, gen_info, namer, query).extract())
        .unzip();

    QueriesInfo {
        query_mod: quote! {
            pub mod #mod_queries {
                #(#mods)*
            }
        }
        .into(),
        query_impls: if impls.is_empty() {
            None
        } else {
            let (impl_database, modifier, type_ds) = if let Some(InterfaceTrait { name }) = interface_trait {
                (quote! { super::#name::#trait_database<#db_lifetime> for }, quote!(), quote!(type #trait_database_type_datastore = #struct_datastore;))
            } else {
                (quote! {}, quote!(pub), quote!())
            };
            Some(
                quote! {
                    impl <#db_lifetime> #impl_database #struct_database<#db_lifetime> {
                        #type_ds
                        #(#modifier #impls)*
                    }
                }
                .into(),
            )
        },
    }
}
