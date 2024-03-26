//! Errors to be displayed by the parser and semantic analysis
//!
//! ```text
//! [PARSE-001] Expected a dodo but found a dada
//!
//!
//!
//! ```
//!

use proc_macro2::{Ident, Span};
use proc_macro_error::{Diagnostic, Level};

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
