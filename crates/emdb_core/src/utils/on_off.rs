use combi::{
    core::{choice, mapsuc},
    macros::choices,
    tokens::{
        basic::{gettoken, matchident, peekident},
        error::error,
        TokenParser,
    },
};
use proc_macro_error2::{Diagnostic, Level};

pub fn on_off() -> impl TokenParser<bool> {
    choices!(
        peekident("on") => mapsuc(matchident("on"), |_| true),
        peekident("off") => mapsuc(matchident("off"), |_| false),
        otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, "Expected `on` or `off`".to_owned()))
    )
}
