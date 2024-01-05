//! combis for better errors

use crate::{core::maperr, Repr};

use super::TokenParser;

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
