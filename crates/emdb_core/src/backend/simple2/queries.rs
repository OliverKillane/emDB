use std::collections::{HashMap, HashSet};

use pulpit::gen::namer::CodeNamer;
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprBlock, Ident, ImplItemFn, ItemEnum, ItemImpl, ItemMod, Path};

use crate::{
    plan,
    utils::misc::{PushMap, PushSet},
};

use super::{
    closures::{generate_application, generate_context, unwrap_context},
    namer::SimpleNamer,
    tables::GeneratedInfo,
    types::generate_scalar_type,
};

fn generate_errors(
    errors: HashMap<Ident, Option<Tokens<Path>>>,
    SimpleNamer {
        mod_queries_mod_query_enum_error,
        ..
    }: &SimpleNamer,
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
    SimpleNamer {
        pulpit:
            CodeNamer {
                struct_window_method_commit,
                struct_window_method_abort,
                ..
            },
        ..
    }: &SimpleNamer,
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
    namer: &SimpleNamer,
    key: plan::Key<plan::Query>,
    plan::Query { name, ctx }: &'imm plan::Query,
) -> QueryMod {
    let SimpleNamer {
        qy_lifetime,
        mod_queries,
        mod_queries_mod_query_enum_error,
        ..
    } = namer;

    let context = lp.get_context(*ctx);
    let return_type = if let Some(ret) = context.get_return_type(lp) {
        let ty = namer.record_name(ret);
        quote!(#ty)
    } else {
        quote!(())
    };

    let params = context.params.iter().map(|(name, ty_key)| {
        let ty = generate_scalar_type(lp, &gen_info.get_types, *ty_key, namer);
        quote!(#name: #ty)
    });

    let top_context_data = generate_context(lp, context, &gen_info.get_types, namer);
    let unwrap_top_data = unwrap_context(context, namer);

    let mut mutated_tables = HashSet::new();
    let mut errors = HashMap::new();

    let query_body = generate_application(
        lp,
        context,
        &quote!(#mod_queries::#name::#mod_queries_mod_query_enum_error).into(),
        &mut PushMap::new(&mut errors),
        &mut PushSet::new(&mut mutated_tables),
        gen_info,
        namer,
    );

    match (
        generate_errors(errors, namer),
        generate_commits(lp, mutated_tables, namer)
    ) {
        (None, None) => {
            QueryMod {
                query_mod: quote! { mod #name {} }.into(),
                query_impl: quote! {
                    pub fn #name<#qy_lifetime>(&#qy_lifetime self, #(#params),* ) -> #return_type {
                        let #unwrap_top_data = #top_context_data;
                        #query_body
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
                    pub fn #name<#qy_lifetime>(&#qy_lifetime mut self, #(#params),* ) -> #return_type {
                        let #unwrap_top_data = #top_context_data;
                        let result = #query_body;
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
                    pub fn #name<#qy_lifetime>(&#qy_lifetime self, #(#params),* ) -> Result<#return_type, #mod_queries::#name::#mod_queries_mod_query_enum_error> {
                        let #unwrap_top_data = #top_context_data;
                        Ok(#query_body)
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
                    pub fn #name<#qy_lifetime>(&#qy_lifetime mut self, #(#params),* ) -> Result<#return_type, #mod_queries::#name::#mod_queries_mod_query_enum_error> {
                        // we catch `?` usage as that short circuits the lambda, not the query method
                        match (|| {
                            let #unwrap_top_data = #top_context_data;
                            Ok(#query_body)
                        })() {
                            Ok(result) => {
                                #commits
                                Ok(result)
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
    pub query_impls: Tokens<ItemImpl>,
}

// TODO: determine error type
// get if an insert for a table has errors, if so, do thingy

pub fn generate_queries<'imm>(
    lp: &'imm plan::Plan,
    gen_info: &GeneratedInfo<'imm>,
    namer: &'imm SimpleNamer,
) -> QueriesInfo {
    let SimpleNamer {
        db_lifetime,
        mod_queries,
        struct_database,
        ..
    } = namer;
    let (mods, impls): (Vec<Tokens<ItemMod>>, Vec<Tokens<ImplItemFn>>) = lp
        .queries
        .iter()
        .map(move |(key, query)| generate_query(lp, gen_info, namer, key, query).extract())
        .unzip();

    QueriesInfo {
        query_mod: quote! {
            pub mod #mod_queries {
                #(#mods)*
            }
        }
        .into(),
        query_impls: quote! {
            impl <#db_lifetime> #struct_database<#db_lifetime> {
                #(#impls)*
            }
        }
        .into(),
    }
}
