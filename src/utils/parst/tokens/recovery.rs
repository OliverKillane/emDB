// TODO: add better recovery, until, past, skip, predicate based.

use proc_macro2::Punct;

use super::*;

pub struct RecoverUptoPunct {
    punct: char,
}

pub fn recoveruptopunct(punct: char) -> RecoverUptoPunct {
    RecoverUptoPunct { punct }
}

impl Recover<TokenIter, SpannedError> for RecoverUptoPunct {
    type C = SpannedCont;

    fn recover(&self, mut input: TokenIter, mut err: SpannedError) -> (TokenIter, Self::C) {
        match input.peek_next() {
            Some(TokenTree::Punct(p)) if p.as_char() == self.punct => {
                err.main = err.main.span_note(
                    p.span(),
                    format!("Ignored by recovering to the next {}", self.punct),
                );
                (input, SpannedCont::from_err(err))
            }
            Some(_) => {
                let tt = input.next().expect("Was already present at last peek_next");
                let start_span = tt.span();
                let mut last_span = start_span;
                loop {
                    match input.peek_next() {
                        Some(TokenTree::Punct(p)) if p.as_char() == self.punct => {
                            last_span = p.span();
                            break;
                        }
                        Some(tt) => {
                            last_span = tt.span();
                            input.next();
                        }
                        None => break,
                    }
                }

                err.main = err.main.span_range_note(
                    SpanRange {
                        first: start_span,
                        last: last_span,
                    },
                    format!("Ignored by recovering to the next {}", self.punct),
                );

                (input, SpannedCont::from_err(err))
            }
            None => (input, SpannedCont::from_err(err)),
        }
    }
}

pub struct RecoverPunct {
    punct: char,
}

pub fn recoverpunct(punct: char) -> RecoverPunct {
    RecoverPunct { punct }
}

impl Recover<TokenIter, SpannedError> for RecoverPunct {
    type C = SpannedCont;

    fn recover(&self, mut input: TokenIter, mut err: SpannedError) -> (TokenIter, Self::C) {
        match input.next() {
            Some(TokenTree::Punct(p)) if p.as_char() == self.punct => {
                err.main = err.main.span_note(
                    p.span(),
                    format!("Ignored by recovering to the next {}", self.punct),
                );
                (input, SpannedCont::from_err(err))
            }
            Some(tt) => {
                let start_span = tt.span();
                let mut last_span = start_span;
                loop {
                    match input.next() {
                        Some(TokenTree::Punct(p)) if p.as_char() == self.punct => {
                            last_span = p.span();
                            break;
                        }
                        Some(tt) => last_span = tt.span(),
                        None => break,
                    }
                }

                err.main = err.main.span_range_note(
                    SpanRange {
                        first: start_span,
                        last: last_span,
                    },
                    format!("Ignored by recovering to the next {}", self.punct),
                );

                (input, SpannedCont::from_err(err))
            }
            None => (input, SpannedCont::from_err(err)),
        }
    }
}

pub struct RecoverImmediate;
pub fn recoverimmediate() -> RecoverImmediate {
    RecoverImmediate
}
impl Recover<TokenIter, SpannedError> for RecoverImmediate {
    type C = SpannedCont;
    fn recover(&self, input: TokenIter, err: SpannedError) -> (TokenIter, Self::C) {
        (input, SpannedCont::from_err(err))
    }
}

pub struct RecoverPunctPred<F: Fn(&Punct) -> bool> {
    pred: F,
}
