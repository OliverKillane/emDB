//! Errors to be displayed by the parser and semantic analysis
//!
//! ```text
//! [PARSE-001] Expected a dodo but found a dada
//!
//!
//!
//! ```
//!

use crate::plan::helpers::*;
use crate::plan::repr::{LogicalPlan, Record, RecordData};
use proc_macro2::{Ident, Span};
use proc_macro_error::{Diagnostic, Level};
use syn::Type;

const BACKEND: &str = "TABLE";
const TABLE: &str = "TABLE";
const QUERY: &str = "QUERY";

fn error_name(section: &str, code: u8) -> String {
    format!("[{}-{:03}]", section, code)
}

fn redefinition_error(
    err_name: String,
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
    redefinition_error(error_name(BACKEND, 0), "backend", def, original_def)
}

pub(super) fn table_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(error_name(TABLE, 1), "table", def, original_def)
}

pub(super) fn table_column_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(error_name(TABLE, 2), "table column", def, original_def)
}

pub(super) fn table_constraint_alias_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(error_name(TABLE, 3), "constraint alias", def, original_def)
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
        diag = diag.span_note(alias.span(), format!("previously defined as {alias} here."))
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
                "".to_owned()
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
                "".to_owned()
            }
        ),
    )
    .span_help(
        table_name.span(),
        "Limit constraints can only be applied once".to_owned(),
    )
}

pub(super) fn query_redefined(def: &Ident, original_def: &Ident) -> Diagnostic {
    redefinition_error(error_name(QUERY, 7), "query", def, original_def)
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
    redefinition_error(error_name(QUERY, 9), "field", def, original_def)
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
    redefinition_error(error_name(QUERY, 13), "query parameter", def, original_def)
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
    lp: &LogicalPlan,
    dt: &RecordData,
    reference: &Ident,
) -> Diagnostic {
    let err_name = error_name(QUERY, 14);

    Diagnostic::spanned(reference.span(), Level::Error, format!(
        "{err_name} Expected a reference to a table for the update in `{reference}`, but got a `{}` instead",
        WithPlan { plan: lp, extended: dt }
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
    lp: &LogicalPlan,
    reference: &Ident,
    prev_span: Span,
    dt: &Record,
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
            WithPlan {
                plan: lp,
                extended: dt
            }
        ),
    )
}

pub(super) fn query_insert_field_rust_type_mismatch(
    lp: &LogicalPlan,
    call: &Ident,
    field: &Ident,
    passed_type: &Type,
    expected_type: &Type,
    prev_span: Span,
) -> Diagnostic {
    let err_name = error_name(QUERY, 18);

    Diagnostic::spanned(call.span(), Level::Error, format!(
        "{err_name} Field `{field}` has type `{:#?}` which does not match the expected type `{:#?}`", passed_type, expected_type
    )).span_note(field.span(), format!("`{field}` defined here"))
    .span_note(prev_span, format!("Input to `{call}` comes from here"))
}

pub(super) fn query_insert_field_type_mismatch(
    lp: &LogicalPlan,
    call: &Ident,
    field: &Ident,
    passed_type: &RecordData,
    expected_type: &Type,
    prev_span: Span,
) -> Diagnostic {
    let err_name = error_name(QUERY, 18);

    Diagnostic::spanned(call.span(), Level::Error, format!(
        "{err_name} Field `{field}` has type `{}` which does not match the expected type `{:#?}`", WithPlan{plan: lp, extended: passed_type}, expected_type
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
    lp: &LogicalPlan,
    call: &Ident,
    field: &Ident,
    dt: &RecordData,
) -> Diagnostic {
    let err_name = error_name(QUERY, 23);

    Diagnostic::spanned(
        field.span(),
        Level::Error,
        format!(
            "{err_name} Field `{field}` is not a reference, but a `{}`",
            WithPlan {
                plan: lp,
                extended: dt
            }
        ),
    )
    .span_help(call.span(), format!("`{field}` "))
}

pub(super) fn query_deref_field_already_exists(named: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 24);

    Diagnostic::spanned(
        named.span(),
        Level::Error,
        format!("{err_name} Field `{named}` already exists",),
    )
    .span_help(
        named.span(),
        format!("Rename `{named}` or remove the existing `{named}` field"),
    )
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
        format!("{err_name} Cannot dereference a rust type `{:#?}`", t),
    )
}

pub(super) fn query_deref_cannot_deref_record(
    lp: &LogicalPlan,
    reference: &Ident,
    t: &Record,
) -> Diagnostic {
    let err_name = error_name(QUERY, 27);
    Diagnostic::spanned(
        reference.span(),
        Level::Error,
        format!(
            "{err_name} Cannot dereference a record `{}`",
            WithPlan {
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

pub(super) fn query_invalid_use(usage: &Ident) -> Diagnostic {
    let err_name = error_name(QUERY, 33);
    Diagnostic::spanned(usage.span(), Level::Error, format!(
        "{err_name} Invalid use of variable `{usage}`",
    )).help(format!(
        "Either use an existing table or variable, or make a new table `table {usage} {{ ... }} @ [ ... ]` or a new variable ` ... |> let {usage}`"
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

pub(super) fn operator_unimplemented(call: &Ident) -> Diagnostic {
    Diagnostic::spanned(
        call.span(),
        Level::Error,
        format!("`{call}` UNIMPLEMENTED!!"),
    )
}
