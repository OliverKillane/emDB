//! # EmQL Error Messages
//! The general error messages produced by emql's semantic analysis (from [`super::sem`] and [`super::operators`])
//! - Each error has a code identified (for easy communication/bug reports)

use super::sem::VarState;
use crate::plan::{self, Key, Plan, RecordType, ScalarType, Table, With};
use itertools::Itertools;
use proc_macro2::{Ident, Span};
use proc_macro_error::{Diagnostic, Level};
use std::collections::HashMap;
use syn::Type;

type ErrCode = usize;

fn emql_error(code: ErrCode, span: Span, message: String) -> Diagnostic {
    Diagnostic::spanned(span, Level::Error, format!("[EMQL-{code}] {message}"))
}

fn redefinition_error(
    err_code: ErrCode,
    def_type: &str,
    def: &Ident,
    original_def: &Ident,
) -> Diagnostic {
    emql_error(
        err_code,
        def.span(),
        format!("Redefinition of {def_type} `{def}`"),
    )
    .span_note(original_def.span(), "Originally defined here".to_string())
    .help(format!("Each {def_type} must have a unique name"))
}

pub fn backend_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(0, "backend", def, original_def)
}

pub fn table_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(1, "table", def, original_def)
}

pub fn table_column_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(2, "table column", def, original_def)
}

pub fn table_constraint_alias_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(3, "constraint alias", def, original_def)
}

pub fn collect_type_alias_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(4, "collect type alias", def, original_def)
}

pub fn table_constraint_duplicate_unique(
    col_name: &Ident,
    method_span: Span,
    prev_alias: &Ident,
) -> Diagnostic {
    emql_error(
        5,
        method_span,
        format!("Duplicate unique constraint on column `{col_name}`"),
    )
    .span_note(
        prev_alias.span(),
        format!("previously defined as {prev_alias} here."),
    )
}

pub fn table_constraint_nonexistent_unique_column(
    alias: &Ident,
    col_name: &Ident,
    table_name: &Ident,
    method_span: Span,
) -> Diagnostic {
    emql_error(6, method_span, format!(
        "Column `{col_name}` does not exist in table `{table_name}`, so cannot apply a unique constraint `{alias}` to it"
    )).span_help(table_name.span(), format!("Apply the unique constraint to an available column in {table_name}"))
}

pub fn table_constraint_duplicate_limit(
    alias: &Ident,
    table_name: &Ident,
    method_span: Span,
) -> Diagnostic {
    emql_error(
        7,
        method_span,
        format!("Duplicate limit constraint `{alias}` on table `{table_name}`"),
    )
    .span_help(
        table_name.span(),
        "Limit constraints can only be applied once".to_string(),
    )
}

pub fn query_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(8, "query", def, original_def)
}

pub fn query_multiple_returns(ret: Span, prev_ret: Span, query: &Ident) -> Diagnostic {
    emql_error(
        9,
        ret,
        format!("Multiple return statements in query `{query}`"),
    )
    .span_help(prev_ret, "Previously returned value here".to_string())
}

pub fn query_operator_field_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(10, "field", def, original_def)
}

pub fn query_stream_single_connection(span: Span, last_span: Span, stream: bool) -> Diagnostic {
    emql_error(
        11,
        span,
        String::from(if stream {
            "Expected a stream, but found a single connector"
        } else {
            "Expected a single, but found a stream connector"
        }),
    )
    .span_note(
        last_span,
        "The previous operator provides the values".to_string(),
    )
    .help(
        if stream {
            "Use a stream connector `|>` instead of single `~>`"
        } else {
            "Use a single connector `~>` instead of stream `|>`"
        }
        .to_string(),
    )
}

pub fn query_no_data_for_next_operator(conn_span: Span, stream: bool, prev_op: Span) -> Diagnostic {
    emql_error(
        12,
        prev_op,
        "No output data provided for next operator".to_string(),
    )
    .span_note(
        conn_span,
        format!(
            "Expected a {} out here",
            if stream {
                "stream of data"
            } else {
                "singe data"
            }
        ),
    )
}

pub fn query_early_return(conn_span: Span, stream: bool, ret_op: Span) -> Diagnostic {
    emql_error(13, ret_op, "Early return statement".to_string()).span_note(
        conn_span,
        format!(
            "Expected a {} out here",
            if stream {
                "stream of data (`|>`)"
            } else {
                "single data (`~>`)"
            }
        ),
    )
}

pub fn query_parameter_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(14, "query parameter", def, original_def)
}

pub fn query_param_ref_table_not_found(query: &Ident, table_ref: &Ident) -> Diagnostic {
    emql_error(
        15,
        table_ref.span(),
        format!("Table `{table_ref}` not found in query `{query}`"),
    )
    .help(format!(
        "Either use a `ref <other table>` or create `table {table_ref} {{...}} @ [...]"
    ))
}

pub fn access_field_missing(call: &Ident, field: &Ident, fields: Vec<&Ident>) -> Diagnostic {
    emql_error(
        16,
        field.span(),
        format!(
            "`{call}` field `{field}` is not found, the available fields are {}",
            fields.iter().join(", ")
        ),
    )
}

pub fn query_expected_reference_type_for_update(
    lp: &Plan,
    dt: &Key<ScalarType>,
    reference: &Ident,
) -> Diagnostic {
    emql_error(
        17,
        reference.span(),
        format!(
        "Expected a reference to a table for the update in `{reference}`, but got a `{}` instead",
        With { plan: lp, extended: dt }
    ),
    )
    .help(format!(
        "Assign a reference to {reference}, or use a different field that is a reference"
    ))
}

pub fn query_cannot_start_with_operator(op: &Ident) -> Diagnostic {
    emql_error(
        18,
        op.span(),
        format!("Cannot start a query expression with operator `{op}`"),
    )
    .help("Instead use operators such as `use ..`, `ref ..`, or `unique(..)`".to_string())
}

pub fn query_update_field_not_in_table(table_name: &Ident, field: &Ident) -> Diagnostic {
    emql_error(
        19,
        field.span(),
        format!("Field `{field}` not found in table `{table_name}` and hence cannot be updated",),
    )
    .span_note(
        table_name.span(),
        format!("The fields that can be updated are present in the `{table_name}` definition"),
    )
}

pub fn query_update_reference_not_present(
    lp: &Plan,
    reference: &Ident,
    prev_span: Span,
    dt: &Key<RecordType>,
) -> Diagnostic {
    emql_error(
        20,
        reference.span(),
        format!("Reference `{reference}` not found in the query"),
    )
    .span_note(
        prev_span,
        format!(
            "The previous operator produces {}",
            With {
                plan: lp,
                extended: dt
            }
        ),
    )
}

pub fn query_insert_field_rust_type_mismatch(
    lp: &Plan,
    call: &Ident,
    field: &Ident,
    passed_type: &Type,
    expected_type: &Type,
    prev_span: Span,
) -> Diagnostic {
    emql_error(21, call.span(), format!(
        "Field `{field}` has type `{passed_type:#?}` which does not match the expected type `{expected_type:#?}`"
    )).span_note(field.span(), format!("`{field}` defined here"))
    .span_note(prev_span, format!("Input to `{call}` comes from here"))
}

pub fn query_insert_field_type_mismatch(
    lp: &Plan,
    call: &Ident,
    field: &Ident,
    passed_type: &Key<RecordType>,
    expected_type: &Type,
    prev_span: Span,
) -> Diagnostic {
    emql_error(
        22,
        call.span(),
        format!(
            "Field `{field}` has type `{}` which does not match the expected type `{:#?}`",
            With {
                plan: lp,
                extended: passed_type
            },
            expected_type
        ),
    )
    .span_note(field.span(), format!("`{field}` defined here"))
    .span_note(prev_span, format!("Input to `{call}` comes from here"))
}

pub fn query_insert_field_missing(
    call: &Ident,
    table_name: &Ident,
    field: &Ident,
    last_span: Span,
) -> Diagnostic {
    emql_error(
        23,
        call.span(),
        format!("Field `{field}` is missing from the insert into table `{table_name}`"),
    )
    .span_note(field.span(), format!("`{field}` defined here"))
    .span_error(
        last_span,
        format!("`{field}` should have been part of the output from here"),
    )
}

pub fn query_insert_extra_field(call: &Ident, field: &Ident, table_name: &Ident) -> Diagnostic {
    emql_error(
        24,
        call.span(),
        format!("Field `{field}` is not a valid field for table `{table_name}`"),
    )
    .span_note(field.span(), format!("`{field}` defined here"))
    .span_note(table_name.span(), format!("`{table_name}` defined here"))
}

pub fn query_nonexistent_table(call: &Ident, table_used: &Ident) -> Diagnostic {
    emql_error(25, table_used.span(), format!(
        "Table `{table_used}` does not exist in the query so cannot be accessed through `{call}`",
    )).help(format!(
        "Either define a `table {table_used} {{...}} @ [...]` or use a different table in `insert(..)`", 
    ))
}

pub fn query_delete_field_not_present(call: &Ident, field: &Ident) -> Diagnostic {
    emql_error(
        26,
        field.span(),
        format!("Field `{field}` not found in the available data"),
    )
    .span_help(
        call.span(),
        format!("`{field}` needs to be accessible here"),
    )
}

pub fn query_delete_field_not_reference(
    lp: &Plan,
    call: &Ident,
    field: &Ident,
    dt: &Key<ScalarType>,
) -> Diagnostic {
    emql_error(
        27,
        field.span(),
        format!(
            "Field `{field}` is not a reference, but a `{}`",
            With {
                plan: lp,
                extended: dt
            }
        ),
    )
    .span_help(call.span(), format!("`{field}` "))
}

pub fn query_deref_field_already_exists(new: &Ident, existing: &Ident) -> Diagnostic {
    emql_error(28, new.span(), format!("Field `{new}` already exists"))
        .span_note(existing.span(), format!("{existing} defined here"))
        .help(format!(
            "Rename `{new}` or remove the existing `{new}` field"
        ))
}

pub fn query_reference_field_missing(reference: &Ident) -> Diagnostic {
    emql_error(
        29,
        reference.span(),
        format!("Field `{reference}` not found in the available data"),
    )
    .span_help(
        reference.span(),
        format!("`{reference}` needs to be accessible here"),
    )
}

pub fn query_deref_cannot_deref_rust_type(reference: &Ident, t: &Type) -> Diagnostic {
    emql_error(
        30,
        reference.span(),
        format!("Cannot dereference a rust type `{t:#?}`"),
    )
}

pub fn query_deref_cannot_deref_record(
    lp: &Plan,
    reference: &Ident,
    t: &Key<RecordType>,
) -> Diagnostic {
    emql_error(
        31,
        reference.span(),
        format!(
            "Cannot dereference a record `{}`",
            With {
                plan: lp,
                extended: t
            }
        ),
    )
}

pub fn query_operator_cannot_come_first(call: &Ident) -> Diagnostic {
    emql_error(
        32,
        call.span(),
        format!("Operator `{call}` cannot be the first operator in a query"),
    )
}

pub fn query_unique_table_not_found(table: &Ident) -> Diagnostic {
    emql_error(
        33,
        table.span(),
        format!("Table `{table}` not found in the query"),
    )
    .help(format!(
        "Either define a `table {table} {{...}} @ [...]` or use a different table in `unique(..)`",
    ))
}

pub fn query_unique_no_field_in_table(field: &Ident, table_name: &Ident) -> Diagnostic {
    emql_error(
        34,
        field.span(),
        format!("Field `{field}` not found in table `{table_name}`"),
    )
    .span_help(
        table_name.span(),
        format!("Add `{field}: ... ,` to {table_name}"),
    )
}

pub fn query_unique_field_is_not_unique(field: &Ident, table_name: &Ident) -> Diagnostic {
    emql_error(35, field.span(), format!("Field `{field}` is not unique in table `{table_name}`"))
    .span_help(
        table_name.span(),
        format!(
            "Add a unique constraint to `{field}` in {table_name} `@ [ ... unique({field}) as ... ]`"
        ),
    )
}

pub fn query_use_variable_already_used(usage: &Ident, created: Span, used: Span) -> Diagnostic {
    emql_error(
        36,
        usage.span(),
        format!("Variable `{usage}` has already been used"),
    )
    .span_error(created, "Was created here".to_string())
    .span_error(used, "And consumed here".to_string())
}

pub fn query_invalid_use(
    usage: &Ident,
    tn: &HashMap<Ident, Key<Table>>,
    vs: &HashMap<Ident, VarState>,
) -> Diagnostic {
    let vars = vs
        .iter()
        .filter_map(|(var, state)| {
            if matches!(state, VarState::Available { .. }) {
                Some(var)
            } else {
                None
            }
        })
        .join(", ");
    let tables = tn.keys().join(", ");
    emql_error(37,usage.span(), format!("Invalid use of variable `{usage}`",))
    .help(format!("Currently available variables are {vars}, and tables {tables}"  ))
    .help(format!(
        "To introduce a new `{usage}` make a new table `table {usage} {{ ... }} @ [ ... ]` or a new variable ` ... |> let {usage}`"
    ))
}

pub fn query_invalid_variable_use(usage: &Ident, vs: &HashMap<Ident, VarState>) -> Diagnostic {
    let vars = vs
        .iter()
        .filter_map(|(var, state)| {
            if matches!(state, VarState::Available { .. }) {
                Some(var)
            } else {
                None
            }
        })
        .join(", ");
    emql_error(
        38,
        usage.span(),
        format!("Invalid use of variable `{usage}`"),
    )
    .help(format!("Currently available variables are {vars}"))
}

pub fn query_let_variable_already_assigned(
    assign: &Ident,
    created: Span,
    used: Option<Span>,
) -> Diagnostic {
    let diag = emql_error(
        39,
        assign.span(),
        format!("Cannot assign to already created variable {assign}"),
    )
    .span_note(created, "Created here".to_string());
    if let Some(used) = used {
        diag.span_note(used, "Used here".to_string())
    } else {
        diag
    }
}

pub fn query_let_variable_shadows_table(assign: &Ident, table: &Ident) -> Diagnostic {
    emql_error(
        55,
        assign.span(),
        format!("variables created by let cannot shadow tables, but `{assign}` does"),
    )
    .span_note(table.span(), "Table defined here".to_string())
}

pub fn query_deref_cannot_deref_bag_type(
    lp: &Plan,
    reference: &Ident,
    t: &Key<RecordType>,
) -> Diagnostic {
    emql_error(
        40,
        reference.span(),
        format!(
            "Cannot dereference a bag of records `{}`",
            With {
                plan: lp,
                extended: t
            }
        ),
    )
}
pub fn query_cannot_return_stream(last: Span, ret: Span) -> Diagnostic {
    emql_error(41, ret, "Cannot return a stream from a query".to_string())
        .span_note(
            last,
            "The previous operator provides the values".to_string(),
        )
        .help("Use a `collect` operator to convert the stream into a bag of records".to_string())
}

pub fn query_table_access_nonexisted_columns(table_name: &Ident, col: &Ident) -> Diagnostic {
    emql_error(
        42,
        col.span(),
        format!("Cannot access {col} as it does not exist in {table_name}"),
    )
    .span_note(table_name.span(), format!("{table_name} defined here"))
}

pub fn query_invalid_record_type(
    lp: &Plan,
    op: &Ident,
    prev: Span,
    expected: &Key<RecordType>,
    found: &Key<RecordType>,
) -> Diagnostic {
    emql_error(
        43,
        op.span(),
        format!(
            "Data type does not match, expected {} but found {}",
            With {
                plan: lp,
                extended: expected
            },
            With {
                plan: lp,
                extended: found
            }
        ),
    )
}

pub fn query_no_cust_type_found(t: &Ident) -> Diagnostic {
    emql_error(44, t.span(), format!("Cannot find type {t}"))
}

pub fn table_query_no_such_field(table: &Ident, t: &Ident) -> Diagnostic {
    emql_error(45, t.span(), format!("no such field `{t}` in `{table}`"))
        .span_note(table.span(), format!("`{table}` defined here"))
}

pub fn query_cannot_append_to_record(new: &Ident, existing: &Ident) -> Diagnostic {
    emql_error(
        46,
        new.span(),
        format!("Cannot append new field `{new}` as it is already defined"),
    )
    .span_note(existing.span(), format!("{existing} defined here"))
}

pub fn sort_field_used_twice(field: &Ident, dup_field: &Ident) -> Diagnostic {
    emql_error(47, field.span(), format!("Field `{field}` is used twice in th sort order, sorts can only sort of each field once"))
    .span_note(dup_field.span(), format!("`{dup_field}` first used here"))
}

pub fn union_requires_at_least_one_input(call: &Ident) -> Diagnostic {
    emql_error(
        48,
        call.span(),
        format!("`{call}` requires at least one input"),
    )
}

pub fn operator_requires_streams(call: &Ident, var: &Ident) -> Diagnostic {
    emql_error(
        49,
        var.span(),
        format!("`{call}` inputs must be streams, but `{var}` is not a stream"),
    )
}

pub fn operator_requires_streams2(call: &Ident) -> Diagnostic {
    emql_error(50, call.span(), format!("`{call}` input must be a stream"))
}

pub fn no_return_in_context(call: &Ident) -> Diagnostic {
    emql_error(
        51,
        call.span(),
        format!("No return from `{call}` is present, needed for the output of `{call}`"),
    )
}

pub fn union_not_same_type(
    lp: &plan::Plan,
    call: &Ident,
    var: &Ident,
    data_type: &plan::Key<plan::RecordType>,
    other_var: &Ident,
    other_data_type: &plan::Key<plan::RecordType>,
) -> Diagnostic {
    emql_error(52, other_var.span(), format!("`{other_var}` has type `{}` but union requires all inputs to be of the same type `{}` (from `{var}`)", plan::With { plan: lp, extended: other_data_type }, plan::With { plan: lp, extended: data_type }))
}

pub fn query_deref_cannot_deref_table_get(
    lp: &plan::Plan,
    reference: &Ident,
    table: plan::Key<plan::Table>,
    field: &plan::RecordField,
) -> Diagnostic {
    let name = &lp.get_table(table).name;
    emql_error(53, reference.span(), format!("Cannot dereference a field directly taken from a table (this is `{name}.{field}`). Try mapping this into a table reference"))
    .note(format!("But what if `{name}.{field}` *is* a table reference? When dereferencing values from a table, the returned value is not the same type as the column, it can be optimised (for example it could be optimised into returning an Rc, a Cow, or a reference to the data). Rather than automatically copying out the value in these cases, it it left to the user to decide how they want to extract this."))
}

pub fn query_combine_extra_field(
    lp: &plan::Plan,
    call: &Ident,
    field: &Ident,
    data_type: &plan::Key<plan::RecordType>,
) -> Diagnostic {
    emql_error(54, field.span(), format!("Field `{field}` is not present in the type for `{call}` (same fields in input as output)"))
    .span_note(field.span(), format!("The type is: `{}`", plan::With { plan: lp, extended: data_type }))
}

pub fn query_combine_missing_field(
    lp: &plan::Plan,
    call: &Ident,
    field: &Ident,
    data_type: &plan::Key<plan::RecordType>,
) -> Diagnostic {
    emql_error(54, field.span(), format!("Field `{field}` is required but not present in the type for `{call}` (same fields in input as output)"))
    .span_note(field.span(), format!("The type is: `{}`", plan::With { plan: lp, extended: data_type }))
}

pub fn query_select_no_field(table: &plan::Table, field: &Ident) -> Diagnostic {
    let cols = table.columns.iter().filter_map(|(rf, _)| if let plan::RecordField::User(id) = rf { Some(id) } else {None}).join(", ");
    
    emql_error(55, field.span(), format!("Field `{field}` is not present in the table `{}`", table.name))
        .span_note(table.name.span(), format!("`{}` defined here with fields: {}", table.name, cols))
}

pub fn query_select_duplicate_field(field_original: &Ident, field: &Ident) -> Diagnostic {
    emql_error(56, field.span(), format!("Duplicate selection on {field}"))
        .span_note(field_original.span(), format!("`{field}` selected here"))
}