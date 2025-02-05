//! For debugging code generation
use proc_macro2::Span;
use proc_macro_error::{Diagnostic, Level};
use std::backtrace::Backtrace;

/// Emit a warning with the specified debug message.
///  - Will de displayed with the span of the entire macro invocation.
pub fn debug(msg: String) {
    let bt = Backtrace::capture();
    Diagnostic::spanned(
        Span::call_site(),
        Level::Warning,
        format!("DEBUG: {msg} \n from {bt}"),
    )
    .emit();
}
