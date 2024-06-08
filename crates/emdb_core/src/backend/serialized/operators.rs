
use quote::{quote, ToTokens};
use quote_debug::Tokens;
use syn::{Expr, Ident, Path, Stmt};

use super::{
    closures::ContextGen,
    namer::{
        boolean_predicate, dataflow_fields, expose_user_fields, new_error, transfer_fields,
        DataFlowNaming, SerializedNamer,
    },
    tables::GeneratedInfo,
    types::generate_record_name,
};
use crate::{
    backend::serialized::closures::generate_application,
    plan::{self, operator_enum},
    utils::misc::{new_id, PushMap, PushSet},
};

#[enumtrait::store(trait_operator_gen)]
pub trait OperatorGen {
    /// Generate the code for the operator
    /// - Needs to update the set of mutated tables
    /// - Adds to the available errors
    ///   NOTE: the behaviour of 'mutates' needs to be the same as for
    ///       [`crate::analysis::mutability`] as that analysis is used for
    ///       generating traits that [`super::serialized`] can implement.
    /// - Adds to the values required for the context.
    #[allow(unused_variables, clippy::too_many_arguments)]
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt>;
}

#[enumtrait::impl_trait(trait_operator_gen for operator_enum)]
impl OperatorGen for plan::Operator {}

// table access
impl OperatorGen for plan::UniqueRef {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            operator_error_parameter,
            mod_tables,
            self_alias,
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
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let unique_reference = namer.transform_field_name(&self.from);
        let new_field = namer.transform_field_name(&self.out);
        let table = lp.get_table(self.table);
        let table_name = &table.name;

        let error_construct = new_error(self_key, error_path, Some(quote!(super::super::#mod_tables::#table_name::#mod_unique::#mod_unique_struct_notfound).into()), errors, namer);

        // TODO: integrate this into the namer somehow?
        let unique_field_access = &table.columns[&self.field]
            .cons
            .unique
            .as_ref()
            .unwrap()
            .alias;

        let transfer_fields = transfer_fields(&input_holding_var, input_record_type, namer);

        let action = quote! {
            let data = #self_alias.#table_name.#unique_field_access(&#input_holding_var.#unique_reference)?;
            Ok(#data_constructor {
                #new_field: data,
                #(#transfer_fields,)*
            })
        };

        let (map_kind, error_kind) = if stream {
            (quote!(map), quote!(error_stream))
        } else {
            (quote!(map_single), quote!(error_single))
        };

        quote! {
            let #holding_var: #dataflow_type = {
                let result = #method_query_operator_alias::#map_kind(
                    #input_holding_var,
                    |#input_holding_var| {
                        #action
                    }
                );
                match #method_query_operator_alias::#error_kind(result) {
                    Ok(val) => val,
                    Err(#operator_error_parameter) => return #error_construct
                }
            };
        }
        .into()
    }
}

impl OperatorGen for plan::ScanRefs {
    fn apply<'imm, 'brw>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            phantom_field,
            self_alias,
            pulpit:
                pulpit::gen::namer::CodeNamer {
                    struct_window_method_scan,
                    ..
                },
            ..
        } = namer;
        let table_name = &lp.get_table(self.table).name;
        let DataFlowNaming {
            holding_var,
            data_constructor,
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let out_ref_name = namer.transform_field_name(&self.out_ref);
        quote! {
            let #holding_var: #dataflow_type = {
                let stream_values = #method_query_operator_alias::consume_stream(
                    #self_alias.#table_name
                        .#struct_window_method_scan()
                        .collect::<Vec<_>>()
                        .into_iter()
                    );
                #method_query_operator_alias::map(
                    stream_values,
                    |value| #data_constructor {
                        #out_ref_name : value,
                        #phantom_field: std::marker::PhantomData
                    }
                )
            };
        }
        .into()
    }
}
impl OperatorGen for plan::DeRef {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            operator_error_parameter,
            self_alias,
            pulpit:
                pulpit::gen::namer::CodeNamer {
                    struct_window_method_get,
                    ..
                },
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
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let table_name = &lp.get_table(self.table).name;
        let deref_field = namer.transform_field_name(&self.reference);
        let new_field = namer.transform_field_name(&self.named);
        let error_variant = namer.operator_error_value_name(self_key);
        let inner_type = generate_record_name(lp, self.named_type, namer);
        let get_value_id = new_id("get_value");

        // In order to expand fields from the old type into the new one
        let transfer_fields_get_struct = transfer_fields(
            &get_value_id,
            lp.get_record_type_conc(self.named_type),
            namer,
        );
        let transfer_fields_input_append = transfer_fields(&input_holding, input_record, namer);

        let (map_kind, error_kind) = if stream {
            (quote!(map), quote!(error_stream))
        } else {
            (quote!(map_single), quote!(error_single))
        };

        if self.unchecked {
            quote!{
                let #holding_var: #dataflow_type = {
                    #method_query_operator_alias::#map_kind(
                        #input_holding,
                        |#input_holding| {
                            match #self_alias.#table_name.#struct_window_method_get(#input_holding.#deref_field) {
                                Ok(#get_value_id) => #data_type {
                                    #new_field: #inner_type {
                                        #(#transfer_fields_get_struct,)*
                                    },
                                    #(#transfer_fields_input_append,)*
                                },
                                Err(_) => unreachable!("This is an unchecked dereference (used internally - e.g. generated by a use")
                            }
                        }
                    )
                };
            }
        } else {
            let error_construct = new_error(self_key, error_path, None, errors, namer);
            quote!{
                let #holding_var: #dataflow_type = {
                    let result = #method_query_operator_alias::#map_kind(
                        #input_holding,
                        |#input_holding| {
                            match #self_alias.#table_name.#struct_window_method_get(#input_holding.#deref_field) {
                                Ok(#get_value_id) => Ok(#data_type {
                                    #new_field: #inner_type {
                                        #(#transfer_fields_get_struct,)*
                                    },
                                    #(#transfer_fields_input_append,)*
                                }),
                                Err(_) => return Err( #error_path::#error_variant )
                            }
                        }
                    );
                    match #method_query_operator_alias::#error_kind(result) {
                        Ok(val) => val,
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
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let closure_val = namer.operator_closure_value_name(self_key);
        let SerializedNamer {
            mod_tables,
            operator_error_parameter,
            method_query_operator_alias,
            phantom_field,
            self_alias,
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
            dataflow_type,
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
        let table_name = &lp.get_table(self.table).name;
        let key_member = namer.transform_field_name(&self.reference);

        let transfer_update_struct = self.mapping.keys().map(|name| {
            let field_name = namer.transform_field_name(name);
            quote!(#field_name: update_struct.#field_name)
        });

        let error_construct = new_error(
            self_key,
            error_path,
            Some(quote!(
                super::super::#mod_tables::#table_name::#mod_update::#update_method::#mod_update_enum_error
            ).into()),
            errors,
            namer
        );

        mutated_tables.push(plan::ImmKey::new(self.table, lp));

        let (map_kind, error_kind) = if stream {
            (quote!(map_seq), quote!(error_stream))
        } else {
            (quote!(map_single), quote!(error_single))
        };

        quote! {
            let #holding_var: #dataflow_type = {
                let results = #method_query_operator_alias::#map_kind(
                    #input_holding,
                    |#input_holding| {
                        // need to clone to avoid borrow issues
                        // TODO: determine how closure clonign affects cloning of internals 
                        let (update_struct, continue_struct) = #closure_val.clone()(#input_holding);

                        match #self_alias.#table_name.#update_method(
                            #mod_tables::#table_name::#mod_update::#update_method::#mod_update_struct_update {
                                #(#transfer_update_struct,)*
                            },
                            continue_struct.#key_member
                        ) {
                            Ok(()) => Ok(continue_struct),
                            Err(#operator_error_parameter) => #error_construct,
                        }
                    }
                );
                #method_query_operator_alias::#error_kind(results)?
            };
        }
        .into()
    }
}
impl OperatorGen for plan::Insert {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            mod_tables,
            operator_error_parameter,
            method_query_operator_alias,
            phantom_field,
            self_alias,
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
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let table_name = &lp.get_table(self.table).name;
        let ref_name = namer.transform_field_name(&self.out_ref);

        mutated_tables.push(plan::ImmKey::new(self.table, lp));

        let (map_kind, error_kind) = if stream {
            (quote!(map_seq), quote!(error_stream))
        } else {
            (quote!(map_single), quote!(error_single))
        };

        let insert_fields = record_type.fields.keys().map(|name| {
            let field_name = namer.transform_field_name(name);
            quote!(#field_name: #input_holding.#field_name)
        });

        let results_internal = if gen_info.insert_can_error[&plan::Idx::new(self.table, lp)] {
            let error_construct = new_error(self_key, error_path, Some(quote!(super::super::#mod_tables::#table_name::#mod_insert::#mod_insert_enum_error).into()), errors, namer);
            quote! {
                {
                    let result = #method_query_operator_alias::#map_kind(
                        #input_holding,
                        |#input_holding| {
                            Ok(#data_constructor {
                                    #ref_name: #self_alias.#table_name.insert(#mod_tables::#table_name::#mod_insert::#mod_insert_struct_insert {
                                    #(#insert_fields,)*
                                })?,
                                #phantom_field: std::marker::PhantomData
                            })
                        }
                    );
                    match #method_query_operator_alias::#error_kind(result) {
                        Ok(val) => val,
                        Err(#operator_error_parameter) => return #error_construct
                    }
                }
            }
        } else {
            quote! {
                #method_query_operator_alias::#map_kind(
                    #input_holding,
                    |#input_holding| {
                        #data_constructor {
                            #ref_name: #self_alias.#table_name.insert(#mod_tables::#table_name::#mod_insert::#mod_insert_struct_insert {
                                #(#insert_fields,)*
                            }),
                            #phantom_field: std::marker::PhantomData
                        }
                    }
                )
            }
        };

        quote! {
            let #holding_var: #dataflow_type = #results_internal;
        }
        .into()
    }
}
impl OperatorGen for plan::Delete {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            mod_tables,
            operator_error_parameter,
            method_query_operator_alias,
            self_alias,
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
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.output, namer);

        mutated_tables.push(plan::ImmKey::new(self.table, lp));

        let (map_kind, error_kind) = if stream {
            (quote!(map_seq), quote!(error_stream))
        } else {
            (quote!(map_single), quote!(error_single))
        };

        let table_name = &lp.get_table(self.table).name;
        let key_member = namer.transform_field_name(&self.reference);
        let error_construct = new_error(
            self_key,
            error_path,
            Some(quote!(super::super::#mod_tables::#table_name::#type_key_error).into()),
            errors,
            namer,
        );

        quote!{
            let #holding_var: #dataflow_type = {
                let result = #method_query_operator_alias::#map_kind(
                    #input_holding,
                    |#input_holding| {
                        match #self_alias.#table_name.#struct_window_method_delete(#input_holding.#key_member) {
                            Ok(()) => Ok(#input_holding),
                            Err(#operator_error_parameter) => #error_construct,
                        }
                    }
                );
                #method_query_operator_alias::#error_kind(result)?
            };
        }.into()
    }
}

// Errors
impl OperatorGen for plan::Assert {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            dataflow_type,
            stream,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let closure_data = namer.operator_closure_value_name(self_key);

        context_vals.push((
            closure_data.clone(),
            (boolean_predicate(lp, &self.assert, self.input, namer).into_token_stream()).into(),
        ));

        let error_construct = new_error(self_key, error_path, None, errors, namer);

        let (map_kind, error_kind) = if stream {
            (quote!(map_seq), quote!(error_stream))
        } else {
            (quote!(map_single), quote!(error_single))
        };

        quote! {
            let #holding_var: #dataflow_type = {
                let result = #method_query_operator_alias::#map_kind(
                    #input_holding,
                    |#input_holding| {
                        if !#closure_data(&#input_holding) {
                            #error_construct
                        } else {
                            Ok(#input_holding)
                        }
                    }
                );
                #method_query_operator_alias::#error_kind(result)?
            };
        }
        .into()
    }
}

impl OperatorGen for plan::Map {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
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
            dataflow_type,
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

        let map_fn = if stream {
            quote!(map)
        } else {
            quote!(map_single)
        };

        quote! {
            let #holding_var: #dataflow_type = #method_query_operator_alias::#map_fn(
                #input_holding,
                #closure_data
            );
        }
        .into()
    }
}
impl OperatorGen for plan::Expand {
    fn apply<'imm, 'brw>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            dataflow_type,
            stream,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let map_fn = if stream {
            quote!(map)
        } else {
            quote!(map_single)
        };

        let expand_field = namer.transform_field_name(&self.field);

        quote! {
            let #holding_var: #dataflow_type = #method_query_operator_alias::#map_fn(
                #input_holding,
                | #input_holding | #input_holding.#expand_field
            );
        }
        .into()
    }
}
impl OperatorGen for plan::Fold {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
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
            dataflow_type,
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

        quote! {
            let #holding_var: #dataflow_type = {
                let (init, update) = #closure_value;
                #method_query_operator_alias::fold(#input_holding, init, update)
            };
        }
        .into()
    }
}
impl OperatorGen for plan::Filter {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let closure_value = namer.operator_closure_value_name(self_key);

        context_vals.push((
            closure_value.clone(),
            (boolean_predicate(lp, &self.predicate, self.input, namer).into_token_stream()).into(),
        ));

        quote!{
            let #holding_var: #dataflow_type = #method_query_operator_alias::filter(#input_holding, #closure_value);
        }.into()
    }
}

impl OperatorGen for plan::Combine {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
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
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let Self {
            left_name,
            right_name,
            ..
        } = self;
        let closure_value = namer.operator_closure_value_name(self_key);

        let field_sets = self.update_fields.iter().map(|(field, expr)| {
            let field_name = namer.transform_field_name(field);
            quote!(#field_name: #expr)
        });

        context_vals.push((
            closure_value.clone(),
            quote! {
                |#left_name: #data_type, #right_name: #data_type|
                    #data_constructor {
                        #(#field_sets,)*
                        #phantom_field: std::marker::PhantomData
                    }
            }
            .into(),
        ));

        quote!{
            let #holding_var: #dataflow_type = #method_query_operator_alias::combine(#input_holding, #closure_value);
        }.into()
    }
}

impl OperatorGen for plan::Sort {
    fn apply<'imm, 'brw>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            dataflow_type,
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

        quote!{
            let #holding_var: #dataflow_type = #method_query_operator_alias::sort(#input_holding, |left, right| {
                #(#comparisons)*
                #order_equal
            });
        }.into()
    }
}
impl OperatorGen for plan::Take {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let closure_value = namer.operator_closure_value_name(self_key);

        let take_expr = &self.limit;

        context_vals.push((
            closure_value.clone(),
            quote! { {let limit: usize = #take_expr; limit} }.into(),
        ));

        quote!{
            let #holding_var: #dataflow_type = #method_query_operator_alias::take(#input_holding, #closure_value);
        }.into()
    }
}
impl OperatorGen for plan::Collect {
    fn apply<'imm, 'brw>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            dataflow_type,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let field = namer.transform_field_name(&self.into);

        // NOTE: Dependent on the collect type being a vector
        //       - Dependency is also present in the `types::generate_scalar_type`
        //         method

        quote!{
            let #holding_var: #dataflow_type = #method_query_operator_alias::consume_single(
                #data_constructor {
                    #field: #method_query_operator_alias::export_stream(#input_holding).collect::<Vec<_>>(),
                    #phantom_field: std::marker::PhantomData
                }
            );
        }.into()
    }
}

impl OperatorGen for plan::Count {
    fn apply<'imm, 'brw>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            dataflow_type,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.output, namer);
        let field = namer.transform_field_name(&self.out_field);

        quote! {
            let #holding_var: #dataflow_type = #method_query_operator_alias::map_single(
                #method_query_operator_alias::count(#input_holding),
                |count|
                #data_constructor {
                    #field: count,
                    #phantom_field: std::marker::PhantomData
                }
            );
        }
        .into()
    }
}

impl OperatorGen for plan::Join {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
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
            dataflow_type,
            data_constructor,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let left_field = namer.transform_field_name(&self.left.identifier);
        let right_field = namer.transform_field_name(&self.right.identifier);

        let joined = match &self.join_kind {
            plan::JoinKind::Inner => match &self.match_kind {
                plan::MatchKind::Cross => {
                    quote! {#method_query_operator_alias::join_cross(#left_hold_var, #right_hold_var)}
                }
                plan::MatchKind::Pred(predicate) => {
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

                    quote! {#method_query_operator_alias::predicate_join(#left_hold_var, #right_hold_var, #join_pred)}
                }
                plan::MatchKind::Equi {
                    left_field,
                    right_field,
                } => {
                    let left_select = namer.transform_field_name(left_field);
                    let right_select = namer.transform_field_name(right_field);
                    quote! {
                        {
                            #method_query_operator_alias::equi_join(#left_hold_var, #right_hold_var, |left: &#data_left| &left.#left_select, |right: &#data_right| &right.#right_select)
                        }
                    }
                }
            },
        };
        quote! {
            let #holding_var: #dataflow_type = #method_query_operator_alias::map(#joined, |(left, right): (#data_left, #data_right)| {
                #data_constructor {
                    #left_field: left,
                    #right_field: right,
                    #phantom_field: std::marker::PhantomData
                }
            });
        }
        .into()
    }
}
impl OperatorGen for plan::Fork {
    fn apply<'imm, 'brw>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            stream,
            ..
        } = dataflow_fields(lp, self.input, namer);

        if self.outputs.is_empty() {
            unreachable!("Cannot generate fork to no outputs")
        } else {

            let fork_op = if stream {
                quote!(fork)
            } else {
                quote!(fork_single)
            };

            let mut outputs = self.outputs.iter();
            let first_output = outputs.next().unwrap();
            let DataFlowNaming { holding_var: first_holding_out, .. } = dataflow_fields(lp, *first_output, namer);

            let (other_outputs_names, other_outputs_fork): (Vec<_>, Vec<_>) = outputs.map(|df_out| {
                let DataFlowNaming { holding_var, .. } = dataflow_fields(lp, *df_out, namer);
                (quote!(#holding_var), quote!(#method_query_operator_alias::#fork_op(&#input_holding)))
            }).unzip();

            quote!{
                let (#(#other_outputs_names,)* #first_holding_out) = (#(#other_outputs_fork,)* #input_holding);
            }
        }.into()
    }
}
impl OperatorGen for plan::Union {
    fn apply<'imm, 'brw>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var,
            dataflow_type,
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
                quote! {
                    #method_query_operator_alias::union(#prev, #var)
                }
            });
            quote! {
                let #holding_var: #dataflow_type = #body;
            }
        }
        .into()
    }
}
impl OperatorGen for plan::Row {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let SerializedNamer {
            method_query_operator_alias,
            phantom_field,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var,
            dataflow_type,
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
            let #holding_var: #dataflow_type = #method_query_operator_alias::consume_single(#data_name);
        }
        .into()
    }
}

impl OperatorGen for plan::Return {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let DataFlowNaming {
            holding_var,
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let return_val = namer.operator_return_value_name(self_key);
        quote! { let #return_val :#dataflow_type = #holding_var; }.into()
    }
}
impl OperatorGen for plan::Discard {
    fn apply<'imm, 'brw>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
        _context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let DataFlowNaming { holding_var, .. } = dataflow_fields(lp, self.input, namer);
        quote! { let _ = #holding_var; }.into()
    }
}

// contexts
impl OperatorGen for plan::GroupBy {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        // scoping out the mutable tables and errors to determine how to generate return and mapping.
        let context_closure_var = namer.operator_closure_value_name(self_key);

        let ContextGen {
            code,
            can_error,
            mutates,
        } = generate_application(
            lp,
            self.inner_ctx,
            error_path,
            errors,
            mutated_tables,
            gen_info,
            namer,
        );

        context_vals.push((context_closure_var.clone(), code.into_token_stream().into()));

        let grouping_field = namer.transform_field_name(&self.group_by);

        let SerializedNamer {
            method_query_operator_alias,
            phantom_field,
            self_alias,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            dataflow_type,
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

        let map_kind = if mutates {
            quote!(map_seq)
        } else {
            quote!(map)
        };

        let final_result = if can_error {
            quote!(#method_query_operator_alias::error_stream(results)?)
        } else {
            quote!(results)
        };

        quote! {
            let #holding_var: #dataflow_type = {
                let split_vars = #method_query_operator_alias::map(
                    #input_holding,
                    |input| {
                        (
                            input.#grouping_field,
                            #inner_data_constructor {
                                #(#inner_fields,)*
                                #phantom_field: std::marker::PhantomData
                            }
                        )
                    }
                );
                let grouped = #method_query_operator_alias::group_by(split_vars);
                let results = #method_query_operator_alias::#map_kind(
                    grouped,
                    |(grouping, inner_stream)| {
                        (#context_closure_var)(#self_alias, grouping, inner_stream)
                    }
                );
                #final_result
            };
        }
        .into()
    }
}

impl OperatorGen for plan::Lift {
    fn apply<'imm, 'brw>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SerializedNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
        mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
        context_vals: &mut Vec<(Ident, Tokens<Expr>)>,
    ) -> Tokens<Stmt> {
        let context_closure_var = namer.operator_closure_value_name(self_key);
        let ContextGen {
            code,
            can_error,
            mutates,
        } = generate_application(
            lp,
            self.inner_ctx,
            error_path,
            errors,
            mutated_tables,
            gen_info,
            namer,
        );
        context_vals.push((context_closure_var.clone(), code.into_token_stream().into()));

        let SerializedNamer {
            method_query_operator_alias,
            self_alias,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var: input_holding,
            stream,
            ..
        } = dataflow_fields(lp, self.input, namer);
        let DataFlowNaming {
            holding_var,
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let map_kind = if stream {
            if mutates {
                quote!(map_seq)
            } else {
                quote!(map)
            }
        } else {
            quote!(map_single)
        };

        let final_result = if can_error {
            let error_map = if stream {
                quote!(error_stream)
            } else {
                quote!(error_single)
            };
            quote!(#method_query_operator_alias::#error_map(results)?)
        } else {
            quote!(results)
        };

        // NOTE: relies on the namer's mapping of operator names leaving user's
        //       field names the same.

        let closure_args = lp
            .get_context(self.inner_ctx)
            .params
            .iter()
            .map(|(id, _)| quote!(lifted.#id));

        quote! {
            let #holding_var: #dataflow_type = {
                let results = #method_query_operator_alias::#map_kind(
                    #input_holding,
                    |lifted| {
                        (#context_closure_var)(#self_alias, #(#closure_args),*)
                    }
                );
                #final_result
            };
        }
        .into()
    }
}
