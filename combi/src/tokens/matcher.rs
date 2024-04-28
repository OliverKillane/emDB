//! Matching tokens using a function.
//!
//! TODO: matching macros to make use of `matches!()``
use crate::{Combi, CombiResult};
use proc_macro2::TokenTree;
use proc_macro_error::{Diagnostic, Level};

use super::{TokenDiagnostic, TokenIter};

#[derive(Clone)]
pub struct Matcher<const PEEK: bool, F>
where
    F: Fn(&TokenTree) -> bool,
{
    match_fn: F,
    repr_str: &'static str,
}

pub fn matcher<const PEEK: bool, F>(match_fn: F, repr_str: &'static str) -> Matcher<PEEK, F>
where
    F: Fn(&TokenTree) -> bool,
{
    Matcher { match_fn, repr_str }
}

impl<const PEEK: bool, F> Combi for Matcher<PEEK, F>
where
    F: Fn(&TokenTree) -> bool,
{
    type Suc = bool;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline(always)]
    fn comp(
        &self,
        mut input: Self::Inp,
    ) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        // TODO: reduce repetition here
        if PEEK {
            if let Some(tk) = input.peek_next() {
                let res = CombiResult::Suc((self.match_fn)(tk));
                (input, res)
            } else {
                let span = *input.cur_span();
                (
                    input,
                    CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                        span,
                        Level::Error,
                        "Unexpected end of input".to_owned(),
                    ))),
                )
            }
        } else if let Some(tk) = input.next() {
            let res = CombiResult::Suc((self.match_fn)(&tk));
            (input, res)
        } else {
            let span = *input.cur_span();
            (
                input,
                CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                    span,
                    Level::Error,
                    "Unexpected end of input".to_owned(),
                ))),
            )
        }
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.repr_str)
    }
}
