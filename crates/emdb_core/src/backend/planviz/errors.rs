use std::io::Error;

use proc_macro_error::{Diagnostic, Level};
use proc_macro2::{Span, Ident};

pub fn expected_options(backend_name: &Ident, opts_repr: &str) -> Diagnostic {
    Diagnostic::spanned(backend_name.span(), Level::Error, format!("No options were provided, but are mandator for {backend_name}. Expected options: {opts_repr}"))
}

pub fn expected_path(backend_name: &Ident) -> Diagnostic {
    Diagnostic::spanned(backend_name.span(), Level::Error, String::from("No `path` option was provided"))
}

pub fn io_error(backend_name: &Ident, path: Span, error: &Error) -> Diagnostic {
    Diagnostic::spanned(path, Level::Error, format!("Failed to create new file for {backend_name} with `{error}`"))
}