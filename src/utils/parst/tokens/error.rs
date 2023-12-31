//! Error helper parser for transforming and generating errors.

use proc_macro_error::Diagnostic;

use crate::utils::parst::core::{mapall, ParseResult};

use super::{Parser, SpannedCont, SpannedError, TokenIter};

pub fn error<O, P: Parser<TokenIter, C = SpannedCont, E = SpannedError>>(
    parser: P,
    err_fn: fn(P::O) -> Diagnostic,
) -> impl Parser<TokenIter, O = O, C = SpannedCont, E = SpannedError> {
    mapall(parser, move |o| {
        ParseResult::Err(SpannedError::from(err_fn(o)))
    })
}
