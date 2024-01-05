//! Recovery parsers for converting errors into continuations/successes

use std::marker::PhantomData;

use super::{TokenDiagnostic, TokenIter};
use crate::{Combi, CombiResult, Repr};
use derive_where::derive_where;
use proc_macro_error::{DiagnosticExt, SpanRange};

/// Recover until the parser succeeds with true
pub fn until<P, T>(parser: P) -> Until<P, T>
where
    P: Combi<
        Inp = TokenIter,
        Out = TokenIter,
        Suc = bool,
        Con = TokenDiagnostic,
        Err = TokenDiagnostic,
    >,
{
    Until {
        parser,
        _marker: PhantomData,
    }
}

#[derive_where(Clone; P)]
pub struct Until<P, T>
where
    P: Combi<
        Inp = TokenIter,
        Out = TokenIter,
        Suc = bool,
        Con = TokenDiagnostic,
        Err = TokenDiagnostic,
    >,
{
    parser: P,
    _marker: PhantomData<T>,
}

impl<P, T> Combi for Until<P, T>
where
    P: Combi<
        Inp = TokenIter,
        Out = TokenIter,
        Suc = bool,
        Con = TokenDiagnostic,
        Err = TokenDiagnostic,
    >,
{
    type Suc = T;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = (TokenDiagnostic, TokenIter);
    type Out = TokenIter;

    fn comp(
        &self,
        (mut err, mut input): Self::Inp,
    ) -> (
        Self::Out,
        crate::CombiResult<Self::Suc, Self::Con, Self::Err>,
    ) {
        let start_span = *input.cur_span();

        loop {
            let (p_out, p_res) = self.parser.comp(input);
            input = p_out;
            match p_res {
                crate::CombiResult::Suc(true) => {
                    err.main = err.main.span_range_note(
                        SpanRange {
                            first: start_span,
                            last: *input.cur_span(),
                        },
                        format!("Ignored by recovering to {}", Repr(&self.parser)),
                    );
                    return (input, CombiResult::Con(err));
                }
                crate::CombiResult::Con(_) | crate::CombiResult::Suc(false) => (),
                // If failing, just propagate the original error
                crate::CombiResult::Err(_) => return (input, CombiResult::Err(err)),
            }
        }
    }

    fn repr(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}
