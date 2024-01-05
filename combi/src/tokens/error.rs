//! combis for better errors

use proc_macro_error::Diagnostic;

use crate::{
    core::{mapall, maperr},
    CombiResult, Repr,
};

use super::{TokenDiagnostic, TokenParser};

pub fn embelisherr<S, P: TokenParser<S>>(parser: P, msg: &'static str) -> impl TokenParser<S> {
    maperr(parser, move |mut e| {
        e.main = e.main.note(String::from(msg));
        e
    })
}

pub fn expectederr<S, P: TokenParser<S>>(parser: P) -> impl TokenParser<S> {
    // TODO: find a way to build &'static str from trait.
    let msg = format!("Expected: {}", Repr(&parser));
    maperr(parser, move |mut e| {
        e.main = e.main.help(msg.clone());
        e
    })
}

pub fn error<S1, S2, P: TokenParser<S1>>(
    parser: P,
    err_fn: impl Fn(S1) -> Diagnostic,
) -> impl TokenParser<S2> {
    mapall(parser, move |o| {
        CombiResult::Err(TokenDiagnostic::from(err_fn(o)))
    })
}
