//! Errors to be displayed by the parser and semantic analysis
//!
//! ```text
//! [PARSE-001] Expected a dodo but found a dada
//!
//!
//!
//! ```
//!

use std::collections::HashMap;

use crate::plan::{Key, Plan, Record, ScalarType, Table, With};
use itertools::Itertools;
use proc_macro2::{Ident, Span};
use proc_macro_error::{Diagnostic, Level};
use syn::Type;

use super::sem::VarState;

const BACKEND: &str = "TABLE";
const TABLE: &str = "TABLE";
const QUERY: &str = "QUERY";

fn error_name(section: &str, code: u8) -> String {
    format!("[{section}-{code:03}]")
}

fn redefinition_error(
    err_name: &str,
    def_type: &str,
    def: &Ident,
    original_def: &Ident,
) -> Diagnostic {
    Diagnostic::spanned(
        def.span(),
        Level::Error,
        format!("{err_name} Redefinition of {def_type} `{def}`"),
    )
    .span_note(original_def.span(), "Originally defined here".to_owned())
    .help(format!("Each {def_type} must have a unique name"))
}

pub(super) fn backend_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(&error_name(BACKEND, 0), "backend", def, original_def)
}

pub(super) fn table_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(&error_name(TABLE, 1), "table", def, original_def)
}

pub(super) fn table_column_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(&error_name(TABLE, 2), "table column", def, original_def)
}

pub(super) fn table_constraint_alias_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(&error_name(TABLE, 3), "constraint alias", def, original_def)
}

pub(super) fn collect_type_alias_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(
        &error_name("type", 1),
        "collect type alias",
        def,
        original_def,
    )
}

pub(super) fn table_constraint_duplicate_unique(
    col_name: &Ident,
    method_span: Span,
    prev_alias: &Option<Ident>,
) -> Diagnostic {
    let err_name = error_name(TABLE, 4);
    let mut diag = Diagnostic::spanned(
        method_span,
        Level::Error,
        format!("{err_name} Duplicate unique constraint on column `{col_name}`"),
    );
    if let Some(alias) = prev_alias {
        diag = diag.span_note(alias.span(), format!("previously defined as {alias} here."));
    }
    diag
}

pub(super) fn table_constraint_nonexistent_unique_column(
    alias: &Option<Ident>,
    col_name: &Ident,
    table_name: &Ident,
    method_span: Span,
) -> Diagnostic {
    let err_name = error_name(TABLE, 5);
    Diagnostic::spanned(
        method_span,
        Level::Error,
        format!(
            "{err_name} Column `{col_name}` does not exist in table `{table_name}`, so cannot apply a unique constraint{} to it", if let Some(alias) = alias {
                format!(" with alias `{alias}`")
            } else {
                String::new()
            }
        ),
    ).span_help(table_name.span(), format!("Apply the unique constraint to an available column in {table_name}"))
}

pub(super) fn table_constraint_duplicate_limit(
    alias: &Option<Ident>,
    table_name: &Ident,
    method_span: Span,
) -> Diagnostic {
    let err_name = error_name(TABLE, 6);
    Diagnostic::spanned(
        method_span,
        Level::Error,
        format!(
            "{err_name} Duplicate limit constraint{} on table `{table_name}`",
            if let Some(alias) = alias {
                format!(" with alias `{alias}`")
            } else {
                String::new()
            }
        ),
    )
    .span_help(
        table_name.span(),
        "Limit constraints can only be applied once".to_owned(),
    )
}

pub(super) fn query_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(&error_name(QUERY, 7), "query", def, original_def)
}

pub(super) fn query_multiple_returns(ret: Span, prev_ret: Span, query: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 8);
    Diagnostic::spanned(
        ret,
        Level::Error,
        format!("{err_name} Multiple return statements in query `{query}`",),
    )
    .span_help(prev_ret, "Previously returned value here".to_owned())
}

pub(super) fn query_operator_field_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(&error_name(QUERY, 9), "field", def, original_def)
}

pub(super) fn query_stream_single_connection(
    span: Span,
    last_span: Span,
    stream: bool,
) -> Diagnostic {
    let err_name = error_name(QUERY, 10);
    Diagnostic::spanned(
        span,
        Level::Error,
        format!(
            "{err_name} {}",
            if stream {
                "Expected a stream, but found a single connector"
            } else {
                "Expected a single, but found a stream connector"
            }
        ),
    )
    .span_note(
        last_span,
        "The previous operator provides the values".to_owned(),
    )
    .help(
        if stream {
            "Use a stream connector `|>` instead of single `~>`"
        } else {
            "Use a single connector `~>` instead of stream `|>`"
        }
        .to_owned(),
    )
}

pub(super) fn query_no_data_for_next_operator(
    conn_span: Span,
    stream: bool,
    prev_op: Span,
) -> Diagnostic {
    let err_name = error_name(QUERY, 11);
    Diagnostic::spanned(
        prev_op,
        Level::Error,
        "No output data provided for next operator".to_owned(),
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

pub(super) fn query_early_return(conn_span: Span, stream: bool, ret_op: Span) -> Diagnostic {
    let err_name = error_name(QUERY, 12);
    Diagnostic::spanned(ret_op, Level::Error, "Early return statement".to_owned()).span_note(
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

pub(super) fn query_parameter_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(&error_name(QUERY, 13), "query parameter", def, original_def)
}

pub(super) fn query_param_ref_table_not_found(query: &Ident, table_ref: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 13);

    Diagnostic::spanned(
        table_ref.span(),
        Level::Error,
        format!("{err_name} Table `{table_ref}` not found in query `{query}`",),
    )
    .help(format!(
        "Either use a `ref <other table>` or create `table {table_ref} {{...}} @ [...]"
    ))
}

pub(super) fn query_expected_reference_type_for_update(
    lp: &Plan,
    dt: &Key<ScalarType>,
    reference: &Ident,
) -> Diagnostic {
    let err_name = error_name(QUERY, 14);

    Diagnostic::spanned(reference.span(), Level::Error, format!(
        "{err_name} Expected a reference to a table for the update in `{reference}`, but got a `{}` instead",
        With { plan: lp, extended: dt }
    )).help(format!("Assign a reference to {reference}, or use a different field that is a reference"))
}

pub(super) fn query_cannot_start_with_operator(op: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 15);

    Diagnostic::spanned(
        op.span(),
        Level::Error,
        format!("{err_name} Cannot start a query expression with operator `{op}`"),
    )
    .help("Instead use operators such as `use ..`, `ref ..`, or `unique(..)`".to_owned())
}

pub(super) fn query_update_field_not_in_table(table_name: &Ident, field: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 16);

    Diagnostic::spanned(
        field.span(),
        Level::Error,
        format!(
        "{err_name} Field `{field}` not found in table `{table_name}` and hence cannot be updated",
    ),
    )
    .span_note(
        table_name.span(),
        format!("The fields that can be updated are present in the `{table_name}` definition"),
    )
}

pub(super) fn query_update_reference_not_present(
    lp: &Plan,
    reference: &Ident,
    prev_span: Span,
    dt: &Key<Record>,
) -> Diagnostic {
    let err_name = error_name(QUERY, 17);

    Diagnostic::spanned(
        reference.span(),
        Level::Error,
        format!("{err_name} Reference `{reference}` not found in the query",),
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

pub(super) fn query_insert_field_rust_type_mismatch(
    lp: &Plan,
    call: &Ident,
    field: &Ident,
    passed_type: &Type,
    expected_type: &Type,
    prev_span: Span,
) -> Diagnostic {
    let err_name = error_name(QUERY, 18);

    Diagnostic::spanned(call.span(), Level::Error, format!(
        "{err_name} Field `{field}` has type `{passed_type:#?}` which does not match the expected type `{expected_type:#?}`"
    )).span_note(field.span(), format!("`{field}` defined here"))
    .span_note(prev_span, format!("Input to `{call}` comes from here"))
}

pub(super) fn query_insert_field_type_mismatch(
    lp: &Plan,
    call: &Ident,
    field: &Ident,
    passed_type: &Key<Record>,
    expected_type: &Type,
    prev_span: Span,
) -> Diagnostic {
    let err_name = error_name(QUERY, 18);

    Diagnostic::spanned(call.span(), Level::Error, format!(
        "{err_name} Field `{field}` has type `{}` which does not match the expected type `{:#?}`", With{plan: lp, extended: passed_type}, expected_type
    )).span_note(field.span(), format!("`{field}` defined here"))
    .span_note(prev_span, format!("Input to `{call}` comes from here"))
}

pub(super) fn query_insert_field_missing(
    call: &Ident,
    table_name: &Ident,
    field: &Ident,
    last_span: Span,
) -> Diagnostic {
    let err_name = error_name(QUERY, 19);

    Diagnostic::spanned(
        call.span(),
        Level::Error,
        format!("{err_name} Field `{field}` is missing from the insert into table `{table_name}`",),
    )
    .span_note(field.span(), format!("`{field}` defined here"))
    .span_error(
        last_span,
        format!("`{field}` should have been part of the output from here"),
    )
}

pub(super) fn query_insert_extra_field(
    call: &Ident,
    field: &Ident,
    table_name: &Ident,
) -> Diagnostic {
    let err_name = error_name(QUERY, 20);

    Diagnostic::spanned(
        call.span(),
        Level::Error,
        format!("{err_name} Field `{field}` is not a valid field for table `{table_name}`",),
    )
    .span_note(field.span(), format!("`{field}` defined here"))
    .span_note(table_name.span(), format!("`{table_name}` defined here"))
}

pub(super) fn query_nonexistent_table(call: &Ident, table_used: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 21);

    Diagnostic::spanned(
        table_used.span(),
        Level::Error,
        format!(
            "{err_name} Table `{table_used}` does not exist in the query so cannot be accessed through `{call}`",
        ),
    ).help(format!(
        "Either define a `table {table_used} {{...}} @ [...]` or use a different table in `insert(..)`", 
    ))
}

pub(super) fn query_delete_field_not_present(call: &Ident, field: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 22);

    Diagnostic::spanned(
        field.span(),
        Level::Error,
        format!("{err_name} Field `{field}` not found in the available data",),
    )
    .span_help(
        call.span(),
        format!("`{field}` needs to be accessible here"),
    )
}

pub(super) fn query_delete_field_not_reference(
    lp: &Plan,
    call: &Ident,
    field: &Ident,
    dt: &Key<ScalarType>,
) -> Diagnostic {
    let err_name = error_name(QUERY, 23);

    Diagnostic::spanned(
        field.span(),
        Level::Error,
        format!(
            "{err_name} Field `{field}` is not a reference, but a `{}`",
            With {
                plan: lp,
                extended: dt
            }
        ),
    )
    .span_help(call.span(), format!("`{field}` "))
}

pub(super) fn query_deref_field_already_exists(new: &Ident, existing: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 24);

    Diagnostic::spanned(
        new.span(),
        Level::Error,
        format!("{err_name} Field `{new}` already exists",),
    )
    .span_note(existing.span(), format!("{existing} defined here"))
    .help(format!(
        "Rename `{new}` or remove the existing `{new}` field"
    ))
}

pub(super) fn query_reference_field_missing(reference: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 25);

    Diagnostic::spanned(
        reference.span(),
        Level::Error,
        format!("{err_name} Field `{reference}` not found in the available data",),
    )
    .span_help(
        reference.span(),
        format!("`{reference}` needs to be accessible here"),
    )
}

pub(super) fn query_deref_cannot_deref_rust_type(reference: &Ident, t: &Type) -> Diagnostic {
    let err_name = error_name(QUERY, 26);
    Diagnostic::spanned(
        reference.span(),
        Level::Error,
        format!("{err_name} Cannot dereference a rust type `{t:#?}`"),
    )
}

pub(super) fn query_deref_cannot_deref_record(
    lp: &Plan,
    reference: &Ident,
    t: &Key<Record>,
) -> Diagnostic {
    let err_name = error_name(QUERY, 27);
    Diagnostic::spanned(
        reference.span(),
        Level::Error,
        format!(
            "{err_name} Cannot dereference a record `{}`",
            With {
                plan: lp,
                extended: t
            }
        ),
    )
}

pub(super) fn query_operator_cannot_come_first(call: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 28);
    Diagnostic::spanned(
        call.span(),
        Level::Error,
        format!("{err_name} Operator `{call}` cannot be the first operator in a query",),
    )
}

pub(super) fn query_unique_table_not_found(table: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 29);
    Diagnostic::spanned(
        table.span(),
        Level::Error,
        format!("{err_name} Table `{table}` not found in the query",),
    )
    .help(format!(
        "Either define a `table {table} {{...}} @ [...]` or use a different table in `unique(..)`",
    ))
}

pub(super) fn query_unique_no_field_in_table(field: &Ident, table_name: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 30);
    Diagnostic::spanned(
        field.span(),
        Level::Error,
        format!("{err_name} Field `{field}` not found in table `{table_name}`",),
    )
    .span_help(
        table_name.span(),
        format!("Add `{field}: ... ,` to {table_name}"),
    )
}

pub(super) fn query_unique_field_is_not_unique(field: &Ident, table_name: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 31);
    Diagnostic::spanned(
        field.span(),
        Level::Error,
        format!("{err_name} Field `{field}` is not unique in table `{table_name}`",),
    )
    .span_help(
        table_name.span(),
        format!(
        "Add a unique constraint to `{field}` in {table_name} `@ [ ... unique({field}) as ... ]`"
    ),
    )
}

pub(super) fn query_use_variable_already_used(
    usage: &Ident,
    created: Span,
    used: Span,
) -> Diagnostic {
    let err_name = error_name(QUERY, 32);
    Diagnostic::spanned(
        usage.span(),
        Level::Error,
        format!("{err_name} Variable `{usage}` has already been used",),
    )
    .span_error(created, "Was created here".to_owned())
    .span_error(used, "And consumed here".to_owned())
}

pub(super) fn query_invalid_use(
    usage: &Ident,
    tn: &HashMap<Ident, Key<Table>>,
    vs: &HashMap<Ident, VarState>,
) -> Diagnostic {
    let err_name = error_name(QUERY, 33);
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
    Diagnostic::spanned(usage.span(), Level::Error, format!(
        "{err_name} Invalid use of variable `{usage}`",
    )).help(format!("Currently available variables are {vars}, and tables {tables}"  ))
    .help(format!(
        "To introduce a new `{usage}` make a new table `table {usage} {{ ... }} @ [ ... ]` or a new variable ` ... |> let {usage}`"
    ))
}

pub(super) fn query_let_variable_already_assigned(
    assign: &Ident,
    created: Span,
    used: Option<Span>,
) -> Diagnostic {
    let err_name = error_name(QUERY, 34);
    let diag = Diagnostic::spanned(
        assign.span(),
        Level::Error,
        format!("{err_name} Cannot assign to already created variable {assign}"),
    )
    .span_note(created, "Created here".to_owned());
    if let Some(used) = used {
        diag.span_note(used, "Used here".to_owned())
    } else {
        diag
    }
}

pub(super) fn query_deref_cannot_deref_bag_type(
    lp: &Plan,
    reference: &Ident,
    t: &Key<Record>,
) -> Diagnostic {
    let err_name = error_name(QUERY, 35);
    Diagnostic::spanned(
        reference.span(),
        Level::Error,
        format!(
            "{err_name} Cannot dereference a bag of records `{}`",
            With {
                plan: lp,
                extended: t
            }
        ),
    )
}
pub(super) fn query_cannot_return_stream(last: Span, ret: Span) -> Diagnostic {
    let err_name = error_name(QUERY, 36);
    Diagnostic::spanned(
        ret,
        Level::Error,
        format!("{err_name} Cannot return a stream from a query",),
    )
    .span_note(last, "The previous operator provides the values".to_owned())
    .help("Use a `collect` operator to convert the stream into a bag of records".to_string())
}

pub(super) fn query_table_access_nonexisted_columns(table_name: &Ident, col: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 37);
    Diagnostic::spanned(
        col.span(),
        Level::Error,
        format!("{err_name} Cannot access {col} as it does not exist in {table_name}"),
    )
    .span_note(table_name.span(), format!("{table_name} defined here"))
}

pub(super) fn query_invalid_record_type(
    lp: &Plan,
    op: &Ident,
    prev: Span,
    expected: &Key<Record>,
    found: &Key<Record>,
) -> Diagnostic {
    let err_name = error_name(QUERY, 38);

    Diagnostic::spanned(
        op.span(),
        Level::Error,
        format!(
            "{err_name} Data type does not match, expected {} but found {}",
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

pub(super) fn query_no_cust_type_found(t: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 39);

    Diagnostic::spanned(
        t.span(),
        Level::Error,
        format!("{err_name} Cannot find type {t}"),
    )
}

pub(super) fn table_query_no_such_field(table: &Ident, t: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 40);

    Diagnostic::spanned(
        t.span(),
        Level::Error,
        format!("{err_name} no such field `{t}` in `{table}`"),
    )
    .span_note(table.span(), format!("`{table}` defined here"))
}

pub(super) fn query_cannot_append_to_record(new: &Ident, existing: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 41);

    Diagnostic::spanned(
        new.span(),
        Level::Error,
        format!("{err_name} cannot append new field `{new}` as it is already defined"),
    )
    .span_note(existing.span(), format!("{existing} defined here"))
}

pub(super) fn sort_field_used_twice(
    field: &Ident,
    dup_field: &Ident,
) -> Diagnostic {
    Diagnostic::spanned(field.span(), Level::Error, format!("Field `{field}` is used twice in th sort order, sorts can only sort of each field once"))
    .span_note(dup_field.span(), format!("`{dup_field}` first used here"))
}

pub(super) fn operator_unimplemented(call: &Ident) -> Diagnostic {
    Diagnostic::spanned(
        call.span(),
        Level::Error,
        format!("`{call}` UNIMPLEMENTED!!"),
    )
}
