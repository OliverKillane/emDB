
use std::iter::{empty, once};

use quote::{quote, ToTokens};
use quote_debug::Tokens;
use syn::{Expr, Ident, Path, Stmt};

use super::{
    closures::{generate_closure_usage, ContextGen}, namer::{
        boolean_predicate, dataflow_fields, expose_user_fields, new_error, transfer_fields,
        DataFlowNaming, SerializedNamer,
    }, stats::{RequiredStats, StatKind}, tables::GeneratedInfo, types::generate_record_name
};
use crate::{
    backend::serialized::closures::generate_application,
    plan::{self, operator_enum, FoldField},
    utils::{misc::{new_id, PushMap}, mut_scope::ScopeHandle},
};



pub enum OperatorImpls {
    Basic,
    Iter,
    Parallel,
    Chunk,
}

impl OperatorImpls {
    pub fn get_paths(&self) -> OperatorImpl {
        match self {
            Self::Basic => OperatorImpl {
                impl_alias: quote!(emdb::dependencies::minister::basic::Basic).into(),
                trait_path: quote!(emdb::dependencies::minister::basic::BasicOps).into(),
            },
            Self::Iter => OperatorImpl {
                impl_alias: quote!(emdb::dependencies::minister::iter::Iter).into(),
                trait_path: quote!(emdb::dependencies::minister::iter::IterOps).into(),
            },
            Self::Parallel => OperatorImpl {
                impl_alias: quote!(emdb::dependencies::minister::parallel::Parallel).into(),
                trait_path: quote!(emdb::dependencies::minister::parallel::ParallelOps).into(),
            },
            Self::Chunk => OperatorImpl {
                impl_alias: quote!(emdb::dependencies::minister::chunk::Chunk).into(),
                trait_path: quote!(emdb::dependencies::minister::chunk::ChunkOps).into(),
            },
        }
    }
}

pub struct OperatorImpl {
    pub impl_alias: Tokens<Path>,
    pub trait_path: Tokens<Path>,
}

#[enumtrait::store(trait_operator_gen)]
pub trait OperatorGen {
    /// Generate the code for the operator
    /// - Needs to update the set of mutated tables
    /// - Adds to the available errors
    ///   NOTE: the behaviour of 'mutates' needs to be the same as for
    ///   [`crate::analysis::mutability`] as that analysis is used for
    ///   generating traits that the serialized backend can implement.
    /// - Adds to the values required for the context.
    #[allow(unused_variables, clippy::too_many_arguments)]
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        op_impl: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt>;
}

#[enumtrait::impl_trait(trait_operator_gen for operator_enum)]
impl OperatorGen for plan::Operator {}

// table access
impl OperatorGen for plan::UniqueRef {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            operator_error_parameter,
            mod_tables,
            pulpit:
                pulpit::gen::namer::CodeNamer {
                    mod_unique,
                    mod_unique_struct_notfound,
                    ..
                },
            ..
        } = namer;

        let DataFlowNaming {
            holding_var: input_holding_var,
            record_type: input_record_type,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            stream,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let unique_reference = namer.transform_field_name(&self.from);
        let new_field = namer.transform_field_name(&self.out);
        
        parent_scope.add_imm(plan::ImmKey::new(self.table, lp));
        let table_param = namer.table_param_name(lp, self.table);
        let table_mod = namer.table_internal_name(lp, self.table);
        let table = lp.get_table(self.table);

        let error_construct = new_error(self_key, error_path, Some(quote!(super::super::#mod_tables::#table_mod::#mod_unique::#mod_unique_struct_notfound).into()), errors, namer);

        // TODO: integrate this into the namer somehow?
        let unique_field_access = &table.columns[&self.field]
            .cons
            .unique
            .as_ref()
            .unwrap()
            .alias;

        let transfer_fields = transfer_fields(&input_holding_var, input_record_type, namer);

        let action = quote! {
            let data = #table_param.#unique_field_access(&#input_holding_var.#unique_reference)?;
            Ok(#data_constructor {
                #new_field: data,
                #(#transfer_fields,)*
            })
        };

        let (map_stats, map_kind, buffer, consume,  error_kind) = if stream {
            (StatKind::Map, quote!(map), quote!(export_buffer), quote!(consume_buffer), quote!(error_stream))
        } else {
            (StatKind::MapSingle, quote!(map_single),  quote!(export_single), quote!(consume_single), quote!(error_single))
        };

        let map_stats_access = namer.access_stat_member(required_stats.add_stat(map_stats));

        quote! {
            let #holding_var = {
                let result = #impl_alias::#consume(#impl_alias::#buffer(#impl_alias::#map_kind(
                    #input_holding_var,
                    |#input_holding_var| {
                        #action
                    },
                    #map_stats_access
                )));
                match #impl_alias::#error_kind(result) {
                    Ok(val) => val,
                    Err(#operator_error_parameter) => return #error_construct
                }
            };
        }
        .into()
    }
}

impl OperatorGen for plan::ScanRefs {
    fn apply<'imm>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            phantom_field,
            pulpit:
                pulpit::gen::namer::CodeNamer {
                    struct_window_method_scan_get,
                    ..
                },
            ..
        } = namer;

        parent_scope.add_imm(plan::ImmKey::new(self.table, lp));
        let table_param = namer.table_param_name(lp, self.table);

        let DataFlowNaming {
            holding_var,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let out_ref_name = namer.transform_field_name(&self.out_ref);
        let map_stats = namer.access_stat_member(required_stats.add_stat(StatKind::Map));
        quote! {
            let #holding_var = {
                let stream_values = #impl_alias::consume_stream(
                    #table_param.#struct_window_method_scan_get()
                    );
                #impl_alias::map(
                    stream_values,
                    |value| #data_constructor {
                        #out_ref_name : value,
                        #phantom_field: std::marker::PhantomData
                    },
                    #map_stats
                )
            };
        }
        .into()
    }
}
impl OperatorGen for plan::DeRef {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            operator_error_parameter,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            record_type: input_record,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            stream,
            data_constructor: data_type,
            ..
        } = dataflow_fields(lp, self.output, namer);

        parent_scope.add_imm(plan::ImmKey::new(self.table, lp));
        let table_param = namer.table_param_name(lp, self.table);
        let deref_field = namer.transform_field_name(&self.reference);
        let new_field = namer.transform_field_name(&self.named);
        let error_variant = namer.operator_error_value_name(self_key);
        let inner_type = generate_record_name(lp, self.named_type, namer);
        let get_value_id = new_id("get_value");
        let get_op_name = namer.pulpit_table_interaction(self_key);

        // In order to expand fields from the old type into the new one
        let transfer_fields_get_struct = transfer_fields(
            &get_value_id,
            lp.get_record_type_conc(self.named_type),
            namer,
        );
        let transfer_fields_input_append = transfer_fields(&input_holding, input_record, namer);

        let (map_stats, map_kind, buffer, consume, error_kind) = if stream {
            (StatKind::Map, quote!(map),quote!(export_buffer), quote!(consume_buffer), quote!(error_stream))
        } else {
            (StatKind::MapSingle, quote!(map_single), quote!(export_single), quote!(consume_single), quote!(error_single))
        };

        let map_stats_access = namer.access_stat_member(required_stats.add_stat(map_stats));

        if self.unchecked {
            quote!{
                let #holding_var = {
                    #impl_alias::#consume(#impl_alias::#buffer(#impl_alias::#map_kind(
                        #input_holding,
                        |#input_holding| {
                            match #table_param.#get_op_name(#input_holding.#deref_field) {
                                Ok(#get_value_id) => #data_type {
                                    #new_field: #inner_type {
                                        #(#transfer_fields_get_struct,)*
                                    },
                                    #(#transfer_fields_input_append,)*
                                },
                                Err(_) => unreachable!("This is an unchecked dereference (used internally - e.g. generated by a use")
                            }
                        },
                        #map_stats_access
                    )))
                };
            }
        } else {
            let error_construct = new_error(self_key, error_path, None, errors, namer);
            quote!{
                let #holding_var = {
                    let result = #impl_alias::#map_kind(
                        #input_holding,
                        |#input_holding| {
                            match #table_param.#get_op_name(#input_holding.#deref_field) {
                                Ok(#get_value_id) => Ok(#data_type {
                                    #new_field: #inner_type {
                                        #(#transfer_fields_get_struct,)*
                                    },
                                    #(#transfer_fields_input_append,)*
                                }),
                                Err(_) => return Err( #error_path::#error_variant )
                            }
                        },
                        #map_stats_access
                    );
                    match #impl_alias::#error_kind(result) {
                        Ok(val) => #impl_alias::#consume(#impl_alias::#buffer(val)),
                        Err(#operator_error_parameter) => return #error_construct
                    }
                };
            }
        }.into()
    }
}
impl OperatorGen for plan::Update {
    // NOTE: We still need to provide a way to keep the values going into the ouput stream.
    //       Our design decisions could be to:
    //       1. Implement some kind of 'split' which lets the user split the update into owned
    //          values passed into the DB, and the values to continue.
    //       2. Let the user decide how to copy and change the values,
    //          and hope the rust compiler removes redundant clones (e.g. value
    //          cloned for insert, but is never used after this)
    // For simplicity of implementation, I have chosen (2.)
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let closure_val = namer.operator_closure_value_name(self_key);
        let SerializedNamer {
            mod_tables,
            operator_error_parameter,
            phantom_field,
            pulpit:
                pulpit::gen::namer::CodeNamer {
                    mod_update,
                    mod_update_struct_update,
                    mod_update_enum_error,
                    ..
                },
            ..
        } = namer;
        let DataFlowNaming {
            data_constructor: input_data_constructor,
            record_type: input_record_type,
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            stream,
            ..
        } = dataflow_fields(lp, self.output, namer);

        {
            // the update expression closure
            let update_type = generate_record_name(lp, self.update_type, namer);
            let update_exprs = self.mapping.iter().map(|(name, expr)| {
                let field_name = namer.transform_field_name(name);
                quote!(#field_name: #expr)
            });
            let args_names = input_record_type
                .fields
                .keys()
                .map(|k| {
                    let field_name = namer.transform_field_name(k);
                    quote!(#field_name)
                })
                .collect::<Vec<_>>();

            context_vals.push((closure_val.clone(), quote! {
                |#input_data_constructor { #(#args_names,)* .. } | {
                    (
                        #update_type { #(#update_exprs,)* #phantom_field: std::marker::PhantomData },
                        #input_data_constructor { #(#args_names,)* #phantom_field: std::marker::PhantomData }
                    )
                }
            }
            .into()));
        }

        let update_method = namer.pulpit_table_interaction(self_key);
        parent_scope.add_mut(plan::ImmKey::new(self.table, lp));
        let table_param = namer.table_param_name(lp, self.table);
        let table_mod = namer.table_internal_name(lp, self.table);

        let key_member = namer.transform_field_name(&self.reference);

        let transfer_update_struct = self.mapping.keys().map(|name| {
            let field_name = namer.transform_field_name(name);
            quote!(#field_name: update_struct.#field_name)
        });

        let error_construct = new_error(
            self_key,
            error_path,
            Some(quote!(
                super::super::#mod_tables::#table_mod::#mod_update::#update_method::#mod_update_enum_error
            ).into()),
            errors,
            namer
        );

        let (map_stats, map_kind, buffer, consume, error_kind) = if stream {
            (StatKind::MapSeq, quote!(map_seq), quote!(export_buffer), quote!(consume_buffer), quote!(error_stream))
        } else {
            (StatKind::MapSingle, quote!(map_single), quote!(export_single), quote!(consume_single), quote!(error_single))
        };

        let map_stats_access = namer.access_stat_member(required_stats.add_stat(map_stats));
        quote! {
            let #holding_var = {
                let results = #impl_alias::#map_kind(
                    #input_holding,
                    |#input_holding| {
                        // NOTE: need to clone to avoid borrow issues
                        // TODO: determine how closure cloning affects cloning of internals 
                        let (update_struct, continue_struct) = #closure_val.clone()(#input_holding);

                        match #table_param.#update_method(
                            #mod_tables::#table_mod::#mod_update::#update_method::#mod_update_struct_update {
                                #(#transfer_update_struct,)*
                            },
                            continue_struct.#key_member
                        ) {
                            Ok(()) => Ok(continue_struct),
                            Err(#operator_error_parameter) => #error_construct,
                        }
                    },
                    #map_stats_access
                );
                #impl_alias::#consume(#impl_alias::#buffer(#impl_alias::#error_kind(results)?))
            };
        }
        .into()
    }
}
impl OperatorGen for plan::Insert {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            mod_tables,
            operator_error_parameter,
            phantom_field,
            pulpit:
                pulpit::gen::namer::CodeNamer {
                    mod_insert,
                    mod_insert_enum_error,
                    mod_insert_struct_insert,
                    ..
                },
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            record_type,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            stream,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.output, namer);

        parent_scope.add_mut(plan::ImmKey::new(self.table, lp));
        let table_param = namer.table_param_name(lp, self.table);
        let table_mod = namer.table_internal_name(lp, self.table);
        let ref_name = namer.transform_field_name(&self.out_ref);

        let (map_stats, map_kind, buffer, consume, error_kind) = if stream {
            (StatKind::Map, quote!(map_seq), quote!(export_buffer), quote!(consume_buffer), quote!(error_stream))
        } else {
            (StatKind::MapSeq, quote!(map_single), quote!(export_single), quote!(consume_single), quote!(error_single))
        };

        let map_stats_access = namer.access_stat_member(required_stats.add_stat(map_stats));

        let insert_fields = record_type.fields.keys().map(|name| {
            let field_name = namer.transform_field_name(name);
            quote!(#field_name: #input_holding.#field_name)
        });

        let results_internal = if gen_info.insert_can_error[&plan::Idx::new(self.table, lp)] {
            let error_construct = new_error(self_key, error_path, Some(quote!(super::super::#mod_tables::#table_mod::#mod_insert::#mod_insert_enum_error).into()), errors, namer);
            quote! {
                {
                    let result = #impl_alias::#map_kind(
                        #input_holding,
                        |#input_holding| {
                            Ok(#data_constructor {
                                    #ref_name: #table_param.insert(#mod_tables::#table_mod::#mod_insert::#mod_insert_struct_insert {
                                    #(#insert_fields,)*
                                })?,
                                #phantom_field: std::marker::PhantomData
                            })
                        },
                        #map_stats_access
                    );
                    match #impl_alias::#error_kind(result) {
                        Ok(val) => val,
                        Err(#operator_error_parameter) => return #error_construct
                    }
                }
            }
        } else {
            quote! {
                #impl_alias::#map_kind(
                    #input_holding,
                    |#input_holding| {
                        #data_constructor {
                            #ref_name: #table_param.insert(#mod_tables::#table_mod::#mod_insert::#mod_insert_struct_insert {
                                #(#insert_fields,)*
                            }),
                            #phantom_field: std::marker::PhantomData
                        }
                    },
                    #map_stats_access
                )
            }
        };

        quote! {
            let #holding_var = #impl_alias::#consume(#impl_alias::#buffer(#results_internal));
        }
        .into()
    }
}
impl OperatorGen for plan::Delete {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            mod_tables,
            operator_error_parameter,
            pulpit:
                pulpit::gen::namer::CodeNamer {
                    type_key_error,
                    struct_window_method_delete,
                    ..
                },
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            stream,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let (map_stats, map_kind, buffer, consume, error_kind) = if stream {
            (StatKind::MapSeq, quote!(map_seq), quote!(export_buffer), quote!(consume_buffer), quote!(error_stream))
        } else {
            (StatKind::MapSingle, quote!(map_single), quote!(export_single), quote!(consume_single), quote!(error_single))
        };

        let map_stats_access = namer.access_stat_member(required_stats.add_stat(map_stats));

        parent_scope.add_mut(plan::ImmKey::new(self.table, lp));
        let table_param = namer.table_param_name(lp, self.table);
        let table_mod = namer.table_internal_name(lp, self.table);
        let key_member = namer.transform_field_name(&self.reference);
        let error_construct = new_error(
            self_key,
            error_path,
            Some(quote!(super::super::#mod_tables::#table_mod::#type_key_error).into()),
            errors,
            namer,
        );

        quote!{
            let #holding_var = {
                let result = #impl_alias::#map_kind(
                    #input_holding,
                    |#input_holding| {
                        match #table_param.#struct_window_method_delete(#input_holding.#key_member) {
                            Ok(()) => Ok(#input_holding),
                            Err(#operator_error_parameter) => #error_construct,
                        }
                    },
                    #map_stats_access
                );
                #impl_alias::#consume(#impl_alias::#buffer(#impl_alias::#error_kind(result)?))
            };
        }.into()
    }
}

// Errors
impl OperatorGen for plan::Assert {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            stream,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let closure_data = namer.operator_closure_value_name(self_key);

        context_vals.push((
            closure_data.clone(),
            (boolean_predicate(lp, &self.assert, self.input, namer).into_token_stream()).into(),
        ));

        let error_construct = new_error(self_key, error_path, None, errors, namer);

        let (stats, all_kind) = if stream {
            (StatKind::All, quote!(all))
        } else {
            (StatKind::Is, quote!(is))
        };

        let stats_access = namer.access_stat_member(required_stats.add_stat(stats));

        quote! {
            let #holding_var = {
                let (result, pass_on) = #impl_alias::#all_kind(
                    #input_holding,
                    #closure_data,
                    #stats_access
                )
                if result {
                    pass_on
                } else {
                    return #error_construct;
                }
            };
        }
        .into()
    }
}

impl OperatorGen for plan::Map {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            data_constructor: input_constructor,
            record_type: input_record_type,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            data_constructor,
            stream,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let closure_data = namer.operator_closure_value_name(self_key);

        let input_fields = expose_user_fields(input_record_type, namer);

        let mapping_fields = self.mapping.iter().map(|(rf, e)| {
            let field_name = namer.transform_field_name(rf);
            quote! {#field_name: #e}
        });

        context_vals.push((
            closure_data.clone(),
            quote! {
                |#input_constructor { #(#input_fields,)* }| {
                    #data_constructor {
                        #(#mapping_fields,)*
                        #phantom_field: std::marker::PhantomData
                    }
                }
            }
            .into(),
        ));

        let (map_stats, map_fn) = if stream {
            (StatKind::Map, quote!(map))
        } else {
            (StatKind::MapSingle, quote!(map_single))
        };

        let map_stats_access = namer.access_stat_member(required_stats.add_stat(map_stats));

        quote! {
            let #holding_var = #impl_alias::#map_fn(
                #input_holding,
                #closure_data,
                #map_stats_access
            );
        }
        .into()
    }
}
impl OperatorGen for plan::Expand {
    fn apply<'imm>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            stream,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let (map_stats, map_fn) = if stream {
            (StatKind::Map, quote!(map))
        } else {
            (StatKind::MapSingle, quote!(map_single))
        };

        let map_stats_access = namer.access_stat_member(required_stats.add_stat(map_stats));

        let expand_field = namer.transform_field_name(&self.field);

        quote! {
            let #holding_var = #impl_alias::#map_fn(
                #input_holding,
                | #input_holding | #input_holding.#expand_field,
                #map_stats_access
            );
        }
        .into()
    }
}
impl OperatorGen for plan::Fold {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            data_constructor: input_data_constructor,
            record_type: input_record_type,
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            data_constructor: acc_constructor,
            data_type: acc_data_type,
            record_type: acc_record_type,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let closure_value = namer.operator_closure_value_name(self_key);

        let (init_fields, update_fields): (Vec<_>, Vec<_>) = self
            .fold_fields
            .iter()
            .map(|(rf, plan::FoldField { initial, update })| {
                let field_name = namer.transform_field_name(rf);
                (
                    quote!(#field_name: #initial),
                    quote!(#field_name: { #update }),
                )
            })
            .unzip();

        let acc_fields = expose_user_fields(acc_record_type, namer);
        let input_fields = expose_user_fields(input_record_type, namer);

        context_vals.push((closure_value.clone(), quote! {
            (
                #acc_constructor {
                    #(#init_fields,)*
                    #phantom_field: std::marker::PhantomData
                },
                |#acc_constructor { #(#acc_fields,)* } : #acc_data_type, #input_data_constructor { #(#input_fields,)* }  | {
                    #acc_constructor {
                        #(#update_fields,)*
                        #phantom_field: std::marker::PhantomData
                    }
                }
            )
        }
        .into()));

        let fold_access_member = namer.access_stat_member(required_stats.add_stat(StatKind::Fold));

        quote! {
            let #holding_var = {
                let (init, update) = #closure_value;
                #impl_alias::fold(#input_holding, init, update, #fold_access_member)
            };
        }
        .into()
    }
}
impl OperatorGen for plan::Filter {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let closure_value = namer.operator_closure_value_name(self_key);

        context_vals.push((
            closure_value.clone(),
            (boolean_predicate(lp, &self.predicate, self.input, namer).into_token_stream()).into(),
        ));

        let filter_access_member = namer.access_stat_member(required_stats.add_stat(StatKind::Filter));

        quote!{
            let #holding_var = #impl_alias::filter(#input_holding, #closure_value, #filter_access_member);
        }.into()
    }
}

impl OperatorGen for plan::Combine {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            data_type,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let Self {
            left_name,
            right_name,
            ..
        } = self;
        let closure_value = namer.operator_closure_value_name(self_key);

        let (field_defaults, field_sets): (Vec<_>, Vec<_>) = self.update_fields.iter().map(|(field, FoldField { initial, update })| {
            let field_name = namer.transform_field_name(field);
            (quote!(#field_name: #initial), quote!(#field_name: #update))
        }).unzip();

        context_vals.push((
            closure_value.clone(),
            quote! {
                (
                    #data_constructor {
                        #(#field_defaults,)*
                        #phantom_field: std::marker::PhantomData
                    },
                    |#left_name: #data_type, #right_name: #data_type|
                        #data_constructor {
                            #(#field_sets,)*
                            #phantom_field: std::marker::PhantomData
                    }
                )
            }
            .into(),
        ));

        let combine_access_member = namer.access_stat_member(required_stats.add_stat(StatKind::Combine));

        quote!{
            let #holding_var = {
                let (alternative, update) = #closure_value;
                #impl_alias::combine(#input_holding, alternative, update, #combine_access_member)   
            };
        }.into()
    }
}

impl OperatorGen for plan::Sort {
    fn apply<'imm>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let order_greater = quote!(std::cmp::Ordering::Greater);
        let order_equal = quote!(std::cmp::Ordering::Equal);
        let order_less = quote!(std::cmp::Ordering::Less);

        let comparisons = self.sort_order.iter().map(|(rf, order)| {
            let (gt_result, lt_result) = match order {
                plan::SortOrder::Asc => (&order_greater, &order_less),
                plan::SortOrder::Desc => (&order_less, &order_greater),
            };
            let field_name = namer.transform_field_name(rf);
            quote! {
                match left.#field_name.cmp(&right.#field_name) {
                    std::cmp::Ordering::Greater => return #gt_result,
                    std::cmp::Ordering::Less => return #lt_result,
                    std::cmp::Ordering::Equal => (),
                }
            }
        });

        let sort_access_member = namer.access_stat_member(required_stats.add_stat(StatKind::Sort));

        quote!{
            let #holding_var = #impl_alias::sort(#input_holding, |left, right| {
                #(#comparisons)*
                #order_equal
            }, #sort_access_member);
        }.into()
    }
}
impl OperatorGen for plan::Take {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let closure_value = namer.operator_closure_value_name(self_key);

        let take_expr = &self.limit;

        context_vals.push((
            closure_value.clone(),
            quote! { {let limit: usize = #take_expr; limit} }.into(),
        ));

        let stats = namer.access_stat_member(required_stats.add_stat(StatKind::Take));

        quote!{
            let #holding_var = #impl_alias::take(#input_holding, #closure_value, #stats);
        }.into()
    }
}
impl OperatorGen for plan::Collect {
    fn apply<'imm>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        _required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let field = namer.transform_field_name(&self.into);

        // NOTE: Dependent on the collect type being a vector
        //       - Dependency is also present in the `types::generate_scalar_type`
        //         method
        //       - Buffer types for operator implementation must be converted into 
        //         vectors

        quote!{
            let #holding_var = #impl_alias::consume_single(
                #data_constructor {
                    #field: #impl_alias::export_buffer(#input_holding).into(),
                    #phantom_field: std::marker::PhantomData
                }
            );
        }.into()
    }
}

impl OperatorGen for plan::Count {
    fn apply<'imm>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let field = namer.transform_field_name(&self.out_field);

        let map_access_member = namer.access_stat_member(required_stats.add_stat(StatKind::MapSingle));
        let count_access_member = namer.access_stat_member(required_stats.add_stat(StatKind::Count));

        quote! {
            let #holding_var = #impl_alias::map_single(
                #impl_alias::count(#input_holding, #count_access_member),
                |count|
                #data_constructor {
                    #field: count,
                    #phantom_field: std::marker::PhantomData
                },
                #map_access_member
            );
        }
        .into()
    }
}

impl OperatorGen for plan::Join {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: left_hold_var,
            data_type: data_left,
            ..
        } = dataflow_fields(lp, self.left.dataflow, namer);
        let DataFlowNaming {
            holding_var: right_hold_var,
            data_type: data_right,
            ..
        } = dataflow_fields(lp, self.right.dataflow, namer);
        let DataFlowNaming {
            holding_var,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let left_field = namer.transform_field_name(&self.left.identifier);
        let right_field = namer.transform_field_name(&self.right.identifier);

        let joined = match &self.join_kind {
            plan::JoinKind::Inner => match &self.match_kind {
                plan::MatchKind::Cross => {
                    let cross_stats = namer.access_stat_member(required_stats.add_stat(StatKind::CrossJoin));
                    quote! {#impl_alias::join_cross(#left_hold_var, #right_hold_var, #cross_stats)}
                }
                plan::MatchKind::Pred(predicate) => {
                    let join_pred_stats = namer.access_stat_member(required_stats.add_stat(StatKind::PredJoin));
                    let join_pred = namer.operator_closure_value_name(self_key);

                    context_vals.push((
                        join_pred.clone(),
                        quote! {
                            |left: &#data_left, right: &#data_right| -> bool {
                                #predicate
                            }
                        }
                        .into(),
                    ));

                    quote! {#impl_alias::predicate_join(#left_hold_var, #right_hold_var, #join_pred, #join_pred_stats)}
                }
                plan::MatchKind::Equi {
                    left_field,
                    right_field,
                } => {
                    let join_equi_stats = namer.access_stat_member(required_stats.add_stat(StatKind::EquiJoin));
                    let left_select = namer.transform_field_name(left_field);
                    let right_select = namer.transform_field_name(right_field);
                    quote! {
                        {
                            #impl_alias::equi_join(#left_hold_var, #right_hold_var, |left: &#data_left| &left.#left_select, |right: &#data_right| &right.#right_select, #join_equi_stats)
                        }
                    }
                }
            },
        };

        let map_stats = namer.access_stat_member(required_stats.add_stat(StatKind::Map));
        quote! {
            let #holding_var = #impl_alias::map(#joined, |(left, right): (#data_left, #data_right)| {
                #data_constructor {
                    #left_field: left,
                    #right_field: right,
                    #phantom_field: std::marker::PhantomData
                }
            }, #map_stats);
        }
        .into()
    }
}
impl OperatorGen for plan::Fork {
    fn apply<'imm>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let DataFlowNaming {
            holding_var: input_holding,
            stream,
            ..
        } = dataflow_fields(lp, self.input, namer);

        if self.outputs.is_empty() {
            unreachable!("Cannot generate fork to no outputs")
        } else {

            let (fork_stat, fork_op) = if stream {
                (StatKind::Fork, quote!(fork))
            } else {
                (StatKind::ForkSingle, quote!(fork_single))
            };

            
            let (mut other_outputs_names, mut other_outputs_fork): (Vec<_>, Vec<_>) = self.outputs.iter().map(|df_out| {
                let DataFlowNaming { holding_var, .. } = dataflow_fields(lp, *df_out, namer);
                let fork_stat_member = namer.access_stat_member(required_stats.add_stat(fork_stat));
                (quote!(#holding_var), quote!(let (#holding_var, temp) = #impl_alias::#fork_op(temp, #fork_stat_member)))
            }).unzip();

            // NOTE: we do not need the final fork, we can just use temp (even if temp 
            //       types are different, we can shadown in the same scope - thanks 
            //       rust 🦀).
            other_outputs_fork.pop();
            let final_name = other_outputs_names.pop();

            quote!{
                let (#(#other_outputs_names,)* #final_name) = {
                    let temp = #input_holding;
                    #(#other_outputs_fork;)*
                    (#(#other_outputs_names,)* temp)
                };
            }
        }.into()
    }
}
impl OperatorGen for plan::Union {
    fn apply<'imm>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let DataFlowNaming {
            holding_var,
            ..
        } = dataflow_fields(lp, self.output, namer);
        if self.inputs.is_empty() {
            unreachable!("Cannot generate union of no data")
        } else {
            let mut inflows = self.inputs.iter();
            let first_input = inflows.next().unwrap();
            let first_holding_in = dataflow_fields(lp, *first_input, namer).holding_var;
            let body = inflows.fold(quote! {#first_holding_in}, |prev, df| {
                let var = dataflow_fields(lp, *df, namer).holding_var;
                let union_stats = namer.access_stat_member(required_stats.add_stat(StatKind::Union));
                quote! {
                    #impl_alias::union(#prev, #var, #union_stats)
                }
            });
            quote! {
                let #holding_var = #body;
            }
        }
        .into()
    }
}
impl OperatorGen for plan::Row {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        OperatorImpl { impl_alias, .. }: &OperatorImpl,
        _required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let data_name = namer.operator_closure_value_name(self_key);

        let data_fields = self.fields.iter().map(|(rf, e)| {
            let member_name = namer.transform_field_name(rf);
            quote!(#member_name: #e)
        });

        context_vals.push((
            data_name.clone(),
            quote! {
                #data_constructor {
                    #(#data_fields,)*
                    #phantom_field: std::marker::PhantomData
                }
            }
            .into(),
        ));

        quote! {
            let #holding_var = #impl_alias::consume_single(#data_name);
        }
        .into()
    }
}

impl OperatorGen for plan::Return {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        _operator_impl: &OperatorImpl,
        _required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let DataFlowNaming {
            holding_var,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let return_val = namer.operator_return_value_name(self_key);
        quote! { let #return_val = #holding_var; }.into()
    }
}
impl OperatorGen for plan::Discard {
    fn apply<'imm>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        _parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        _operator_impl: &OperatorImpl,
        _required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let DataFlowNaming { holding_var, .. } = dataflow_fields(lp, self.input, namer);
        quote! { let _ = #holding_var; }.into()
    }
}

// contexts
impl OperatorGen for plan::GroupBy {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        operator_impl: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        // scoping out the mutable tables and errors to determine how to generate return and mapping.
        let context_closure_var = namer.operator_closure_value_name(self_key);
        let OperatorImpl {impl_alias, ..} = operator_impl;
        let ContextGen {
            code,
            can_error,
            scope,
        } = generate_application(
            lp,
            self.inner_ctx,
            error_path,
            errors,
            parent_scope,
            gen_info,
            namer,
            operator_impl,
            required_stats,
        );

        context_vals.push((context_closure_var.clone(), code.into_token_stream().into()));

        let grouping_field = namer.transform_field_name(&self.group_by);

        let SerializedNamer {
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let DataFlowNaming {
            data_constructor: inner_data_constructor,
            record_type: inner_record_type,
            ..
        } = dataflow_fields(lp, self.stream_in, namer);

        let inner_fields = inner_record_type.fields.keys().map(|rf| {
            let field_name = namer.transform_field_name(rf);
            quote!(#field_name: input.#field_name)
        });

        let (map_stats, map_kind) = if scope.mutates() {
            (StatKind::MapSeq, quote!(map_seq))
        } else {
            (StatKind::Map, quote!(map))
        };

        let map_stats = namer.access_stat_member(required_stats.add_stat(map_stats));

        let final_result = if can_error {
            quote!(#impl_alias::error_stream(results)?)
        } else {
            quote!(results)
        };

        // TODO: Further opportunity to optimise here
        //       - Allowing non-mutating, non-erroring to be computed inside the groupby?
        //       - No longer need to materialise output if we can stream through

        let group_by_stats = namer.access_stat_member(required_stats.add_stat(StatKind::GroupBy));

        let args = generate_closure_usage(lp, namer, 
            once(quote!(grouping)),
            once(quote!(inner_stream)),
            &scope
        );

        quote! {
            let #holding_var = {
                let grouped = #impl_alias::group_by(
                    #input_holding,
                    |input| {
                        (
                            input.#grouping_field,
                            #inner_data_constructor {
                                #(#inner_fields,)*
                                #phantom_field: std::marker::PhantomData
                            }
                        )
                    },
                    #group_by_stats
                );
                let results = #impl_alias::#map_kind(
                    grouped,
                    |(grouping, inner_stream)| {
                        (#context_closure_var)#args
                    },
                    #map_stats
                );
                #final_result
            };
        }
        .into()
    }
}

impl OperatorGen for plan::Lift {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
        parent_scope: &mut ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
        operator_impl: &OperatorImpl,
        required_stats: &mut RequiredStats,
    ) -> Tokens<Stmt> {
        let context_closure_var = namer.operator_closure_value_name(self_key);
        let OperatorImpl {impl_alias, ..} = operator_impl;
        let ContextGen {
            code,
            can_error,
            scope,
        } = generate_application(
            lp,
            self.inner_ctx,
            error_path,
            errors,
            parent_scope,
            gen_info,
            namer,
            operator_impl,
            required_stats,
        );
        context_vals.push((context_closure_var.clone(), code.into_token_stream().into()));

        let DataFlowNaming {
            holding_var: input_holding,
            stream,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let (map_stats, map_kind) = if stream {
            if scope.mutates() {
                (StatKind::MapSeq, quote!(map_seq))
            } else {
                (StatKind::Map, quote!(map))
            }
        } else {
            (StatKind::MapSingle, quote!(map_single))
        };

        let map_stats_member = namer.access_stat_member(required_stats.add_stat(map_stats));

        let final_result = if can_error {
            let error_map = if stream {
                quote!(error_stream)
            } else {
                quote!(error_single)
            };
            quote!(#impl_alias::#error_map(results)?)
        } else {
            quote!(results)
        };

        // NOTE: relies on the namer's mapping of operator names leaving user's
        //       field names the same.
        let closure_param_args = lp
            .get_context(self.inner_ctx)
            .params
            .iter()
            .map(|(id, _)| quote!(lifted.#id));

        let args = generate_closure_usage(
            lp,
            namer,
            closure_param_args,
            empty::<Ident>(),
            &scope,
        );

        quote! {
            let #holding_var = {
                let results = #impl_alias::#map_kind(
                    #input_holding,
                    |lifted| {
                        (#context_closure_var)#args
                    },
                    #map_stats_member
                );
                #final_result
            };
        }
        .into()
    }
}
