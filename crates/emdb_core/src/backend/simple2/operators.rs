use std::collections::HashMap;

use quote::quote;
use quote_debug::Tokens;
use syn::{Expr, Ident, Path, Stmt, Type};

use super::{
    namer::{dataflow_fields, new_error, new_id, transfer_fields, DataFlowNaming, SimpleNamer},
    tables::GeneratedInfo,
    types::generate_record_name,
};
use crate::{
    plan::{self, operator_enum},
    utils::misc::{PushMap, PushSet},
};

// TODO: continue with operators
//       - determine if an operator mutates a table?
//         -> can use parallel access for contexts

#[enumtrait::store(trait_operator_gen)]
pub trait OperatorGen {
    /// Generate the data needed that captures from query parameters
    #[allow(unused_variables)]
    fn closure_data<'imm>(
        &self,
        lp: &'imm plan::Plan,
        get_types: &HashMap<plan::Idx<'imm, plan::Table>, HashMap<Ident, Tokens<Type>>>,
        namer: &SimpleNamer,
    ) -> Tokens<Expr> {
        quote! { () }.into()
    }

    /// Generate the code for the operator
    /// - Needs to update the set of mutated tables
    /// - Adds to the available errors
    #[allow(unused_variables, clippy::too_many_arguments)]
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SimpleNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<Ident, Option<Tokens<Path>>>,
        mutated_tables: &mut PushSet<plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
    ) -> Tokens<Stmt> {
        quote! { let _ = (); }.into()
    }
}

#[enumtrait::impl_trait(trait_operator_gen for operator_enum)]
impl OperatorGen for plan::Operator {}

// table access
impl OperatorGen for plan::UniqueRef {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SimpleNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
    ) -> Tokens<Stmt> {
        let SimpleNamer {
            method_query_operator_alias,
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
            let data = self.#table_name.#unique_field_access(&#input_holding_var.#unique_reference)?;
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
    fn apply<'imm>(
        &self,
        _self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SimpleNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
    ) -> Tokens<Stmt> {
        let SimpleNamer {
            method_query_operator_alias,
            phantom_field,
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
        quote!{
            let #holding_var: #dataflow_type = {
                let stream_values = #method_query_operator_alias::consume_stream(self.#table_name.#struct_window_method_scan().collect::<Vec<_>>().into_iter());
                #method_query_operator_alias::map(stream_values, |value| #data_constructor { #out_ref_name : value, #phantom_field: std::marker::PhantomData })
            };
        }.into()
    }
}
impl OperatorGen for plan::DeRef {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SimpleNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
    ) -> Tokens<Stmt> {
        let SimpleNamer {
            method_query_operator_alias,
            operator_error_parameter,
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
        let error_construct = new_error(self_key, error_path, None, errors, namer);
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

        quote!{
            let #holding_var: #dataflow_type = {
                let result = #method_query_operator_alias::#map_kind(
                    #input_holding,
                    |#input_holding| {
                        match self.#table_name.#struct_window_method_get(#input_holding.#deref_field) {
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

    fn closure_data<'imm>(
        &self,
        lp: &'imm plan::Plan,
        _get_types: &HashMap<plan::Idx<'imm, plan::Table>, HashMap<Ident, Tokens<Type>>>,
        namer: &SimpleNamer,
    ) -> Tokens<Expr> {
        let SimpleNamer { phantom_field, .. } = namer;
        let DataFlowNaming {
            data_constructor,
            record_type,
            ..
        } = dataflow_fields(lp, self.input, namer);

        // the update type name
        let update_type = generate_record_name(lp, self.update_type, namer);
        let update_exprs = self.mapping.iter().map(|(name, expr)| {
            let field_name = namer.transform_field_name(name);
            quote!(#field_name: #expr)
        });
        let args_names = record_type
            .fields
            .keys()
            .map(|k| {
                let field_name = namer.transform_field_name(k);
                quote!(#field_name)
            })
            .collect::<Vec<_>>();

        quote! {
            |#data_constructor { #(#args_names,)* .. } | {
                (
                    #update_type { #(#update_exprs,)* #phantom_field: std::marker::PhantomData },
                    #data_constructor { #(#args_names,)* #phantom_field: std::marker::PhantomData }
                )
            }
        }
        .into()
    }

    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SimpleNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<Ident, Option<Tokens<Path>>>,
        mutated_tables: &mut PushSet<plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
    ) -> Tokens<Stmt> {
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

        let SimpleNamer {
            mod_tables,
            operator_error_parameter,
            method_query_operator_alias,
            pulpit:
                pulpit::gen::namer::CodeNamer {
                    mod_update,
                    mod_update_struct_update,
                    mod_update_enum_error,
                    ..
                },
            ..
        } = namer;

        let update_method = namer.pulpit_table_interaction(self_key);
        let table_name = &lp.get_table(self.table).name;
        let key_member = namer.transform_field_name(&self.reference);

        let transfer_update_struct = self.mapping.keys().map(|name| {
            let field_name = namer.transform_field_name(name);
            quote!(#field_name: update_struct.#field_name)
        });

        let error_construct = new_error(self_key, error_path, Some(quote!(super::super::#mod_tables::#table_name::#mod_update::#update_method::#mod_update_enum_error).into()), errors, namer);

        mutated_tables.push(plan::ImmKey::new(self.table, lp));

        let closure_val = namer.operator_closure_value_name(self_key);

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
                        let (update_struct, continue_struct) = #closure_val(#input_holding);

                        match self.#table_name.#update_method(
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
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SimpleNamer,
        error_path: &Tokens<Path>,
        errors: &mut PushMap<Ident, Option<Tokens<Path>>>,
        mutated_tables: &mut PushSet<plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
    ) -> Tokens<Stmt> {
        let SimpleNamer {
            mod_tables,
            operator_error_parameter,
            method_query_operator_alias,
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
                                    #ref_name: self.#table_name.insert(#mod_tables::#table_name::#mod_insert::#mod_insert_struct_insert {
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
                            #ref_name: self.#table_name.insert(#mod_tables::#table_name::#mod_insert::#mod_insert_struct_insert {
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
impl OperatorGen for plan::Delete {}

// Errors
impl OperatorGen for plan::Assert {}

// Pure
impl OperatorGen for plan::Map {}
impl OperatorGen for plan::Expand {}
impl OperatorGen for plan::Fold {}
impl OperatorGen for plan::Filter {}
impl OperatorGen for plan::Sort {}
impl OperatorGen for plan::Take {}
impl OperatorGen for plan::Collect {}
impl OperatorGen for plan::Join {}
impl OperatorGen for plan::Fork {}
impl OperatorGen for plan::Union {}
impl OperatorGen for plan::Row {
    fn closure_data<'imm>(
        &self,
        lp: &'imm plan::Plan,
        _get_types: &HashMap<plan::Idx<'imm, plan::Table>, HashMap<Ident, Tokens<Type>>>,
        namer: &SimpleNamer,
    ) -> Tokens<Expr> {
        let SimpleNamer { phantom_field, .. } = namer;

        let DataFlowNaming {
            data_constructor, ..
        } = dataflow_fields(lp, self.output, namer);

        let data_fields = self.fields.iter().map(|(rf, e)| {
            let member_name = namer.transform_field_name(rf);
            quote!(#member_name: #e)
        });
        quote! {
            #data_constructor {
                #(#data_fields,)*
                #phantom_field: std::marker::PhantomData
            }
        }
        .into()
    }

    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SimpleNamer,
        _error_path: &Tokens<Path>,
        _errors: &mut PushMap<Ident, Option<Tokens<Path>>>,
        _mutated_tables: &mut PushSet<plan::ImmKey<'imm, plan::Table>>,
        _gen_info: &GeneratedInfo<'imm>,
    ) -> Tokens<Stmt> {
        let SimpleNamer {
            method_query_operator_alias,
            ..
        } = namer;
        let DataFlowNaming {
            holding_var,
            dataflow_type,
            ..
        } = dataflow_fields(lp, self.output, namer);

        let data_name = namer.operator_closure_value_name(self_key);

        quote! {
            let #holding_var: #dataflow_type = #method_query_operator_alias::consume_single(#data_name);
        }
        .into()
    }
}

// control flow
impl OperatorGen for plan::Return {}
impl OperatorGen for plan::Discard {}

// contexts
impl OperatorGen for plan::GroupBy {}
impl OperatorGen for plan::ForEach {}
