//! For debugging code generation
use proc_macro2::Span;
use proc_macro_error2::{Diagnostic, Level};
use std::backtrace::Backtrace;

/// Emit a warning with the specified debug message.
///  - Will de displayed with the span of the entire macro invocation.
///  - Requires a backtrace (set in crates/.cargo/config.toml)
#[track_caller] // TODO: Do we want this?
pub fn debug(msg: String) {
    let bt = Backtrace::capture();
    Diagnostic::spanned(
        Span::call_site(),
        Level::Warning,
        format!("DEBUG: {msg} \n from {bt}"),
    )
    .emit();
}
