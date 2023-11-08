use std::collections::LinkedList;

use super::core::{seq, ConComb, ErrComb, ParseResult, Parser, Recover, Seq};

use proc_macro2::{token_stream::IntoIter, Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use proc_macro_error::Diagnostic;

// ident, group, punct, match ident, match group, match punct, term

/// A wrapper for the tokentree iterator that allows for a 1-tokentree peek
pub struct TokenIter {
    next: Option<TokenTree>,
    iter: IntoIter,
    prev_span: Option<Span>,
    prev_span2: Option<Span>,
}

impl TokenIter {
    pub fn from(ts: TokenStream) -> Self {
        let mut iter = ts.into_iter();

        Self {
            next: iter.next(),
            iter: iter,
            prev_span: None,
            prev_span2: None,
        }
    }

    fn next(&mut self) -> Option<TokenTree> {
        let mut tk = self.iter.next();
        std::mem::swap(&mut self.next, &mut tk);
        self.prev_span2 = self.prev_span;
        self.prev_span = tk.as_ref().map(|t| t.span());
        tk
    }

    fn peek_next(&self) -> &Option<TokenTree> {
        &self.next
    }

    fn last_span(&self) -> &Option<Span> {
        &self.prev_span2
    }
}

pub struct TokenError {
    main: Diagnostic,
    prev: LinkedList<Diagnostic>,
}

impl TokenError {
    fn from(main: Diagnostic) -> Self {
        Self {
            main,
            prev: LinkedList::new(),
        }
    }
}
pub struct TokenCont(LinkedList<Diagnostic>);

impl<O> ConComb<O, Self> for TokenCont {
    fn combine_out(self, out: O) -> Self {
        // entirely ignore the output - we don't care about it
        self
    }

    fn combine_con(mut self, mut con: Self) -> Self {
        self.0.append(&mut con.0);
        self
    }
}

impl ErrComb<TokenCont> for TokenError {
    fn combine_con(self, mut con: TokenCont) -> Self {
        Self {
            main: self.main,
            prev: {
                let mut prev = self.prev;
                prev.append(&mut con.0);
                prev
            },
        }
    }
}

pub struct GetIdent;

pub fn getident() -> GetIdent {
    GetIdent
}

impl Parser<TokenIter> for GetIdent {
    type O = Ident;
    type C = TokenCont;
    type E = TokenError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.next() {
            Some(TokenTree::Ident(i)) => (input, ParseResult::Suc(i)),
            Some(tt) => (
                input,
                ParseResult::Err(TokenError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected identifier, found {}", tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    ParseResult::Err(TokenError::from(Diagnostic::spanned(
                        span,
                        proc_macro_error::Level::Error,
                        format!("Expected identifier, found nothing"),
                    ))),
                )
            }
        }
    }
}

struct Terminal;

pub fn terminal() -> Terminal {
    Terminal
}

impl Parser<TokenIter> for Terminal {
    type O = ();
    type C = TokenCont;
    type E = TokenError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        if let Some(tt) = input.next() {
            (
                input,
                ParseResult::Err(TokenError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected end of input"),
                ))),
            )
        } else {
            (input, ParseResult::Suc(()))
        }
    }
}

pub struct InGroup<P: Parser<TokenIter, E = TokenError, C = TokenCont>> {
    delim: Delimiter,
    parser: Seq<TokenIter, TokenError, TokenCont, P, Terminal>,
}

pub fn ingroup<P: Parser<TokenIter, E = TokenError, C = TokenCont>>(
    delim: Delimiter,
    p: P,
) -> InGroup<P> {
    // let y = seq(p, terminal());
    // y.parse(todo!());

    InGroup {
        delim,
        parser: seq(p, terminal()),
    }
}

impl<P: Parser<TokenIter, E = TokenError, C = TokenCont>> Parser<TokenIter> for InGroup<P> {
    type O = P::O;
    type C = P::C;
    type E = TokenError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.next() {
            Some(TokenTree::Group(g)) => {
                if g.delimiter() == self.delim {
                    let (_, res) = self.parser.parse(TokenIter::from(g.stream()));
                    (
                        input,
                        match res {
                            ParseResult::Err(e) => ParseResult::Err(e),
                            ParseResult::Con(c) => ParseResult::Con(c),
                            ParseResult::Suc((res, _)) => ParseResult::Suc(res),
                        },
                    )
                } else {
                    (
                        input,
                        ParseResult::Err(TokenError::from(Diagnostic::spanned(
                            g.span(),
                            proc_macro_error::Level::Error,
                            format!(
                                "Expected group with delimiter {:?}, found {:?}",
                                self.delim,
                                g.delimiter()
                            ),
                        ))),
                    )
                }
            }
            Some(tt) => (
                input,
                ParseResult::Err(TokenError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected group bracketed by {:?}", self.delim),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    ParseResult::Err(TokenError::from(Diagnostic::spanned(
                        span,
                        proc_macro_error::Level::Error,
                        format!(
                            "Expected a following group bracketed by {:?}, but no input present",
                            self.delim
                        ),
                    ))),
                )
            }
        }
    }
}

pub struct MatchIdent {
    text: &'static str,
}

pub fn matchident(text: &'static str) -> MatchIdent {
    MatchIdent { text }
}

impl Parser<TokenIter> for MatchIdent {
    type O = ();
    type C = TokenCont;
    type E = TokenError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.next() {
            Some(TokenTree::Ident(i)) => {
                if i.to_string() == self.text {
                    (input, ParseResult::Suc(()))
                } else {
                    (
                        input,
                        ParseResult::Err(TokenError::from(Diagnostic::spanned(
                            i.span(),
                            proc_macro_error::Level::Error,
                            format!("Expected identifier {}, found {}", self.text, i),
                        ))),
                    )
                }
            }
            Some(tt) => (
                input,
                ParseResult::Err(TokenError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected identifier {}, found {}", self.text, tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    ParseResult::Err(TokenError::from(Diagnostic::spanned(
                        span,
                        proc_macro_error::Level::Error,
                        format!("Expected identifier {}, found nothing", self.text),
                    ))),
                )
            }
        }
    }
}

pub struct MatchPunct {
    punct: char,
}

pub fn matchpunct(punct: char) -> MatchPunct {
    MatchPunct { punct }
}

impl Parser<TokenIter> for MatchPunct {
    type O = ();
    type C = TokenCont;
    type E = TokenError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.next() {
            Some(TokenTree::Punct(p)) => {
                if p.as_char() == self.punct {
                    (input, ParseResult::Suc(()))
                } else {
                    (
                        input,
                        ParseResult::Err(TokenError::from(Diagnostic::spanned(
                            p.span(),
                            proc_macro_error::Level::Error,
                            format!("Expected punct {}, found {}", self.punct, p),
                        ))),
                    )
                }
            }
            Some(tt) => (
                input,
                ParseResult::Err(TokenError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected punct {}, found {}", self.punct, tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    ParseResult::Err(TokenError::from(Diagnostic::spanned(
                        span,
                        proc_macro_error::Level::Error,
                        format!("Expected punct {}, found nothing", self.punct),
                    ))),
                )
            }
        }
    }
}

pub struct PunctOrEnd {
    punct: char,
}

pub fn punctorend(punct: char) -> PunctOrEnd {
    PunctOrEnd { punct }
}

impl Parser<TokenIter> for PunctOrEnd {
    type O = bool;
    type C = TokenCont;
    type E = TokenError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.next() {
            Some(TokenTree::Punct(p)) => {
                if p.as_char() == self.punct {
                    (input, ParseResult::Suc(true))
                } else {
                    (
                        input,
                        ParseResult::Err(TokenError::from(Diagnostic::spanned(
                            p.span(),
                            proc_macro_error::Level::Error,
                            format!("Expected punct '{}', found '{}'", self.punct, p.as_char()),
                        ))),
                    )
                }
            }
            Some(tt) => (
                input,
                ParseResult::Err(TokenError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected punct '{}', found '{}'", self.punct, tt),
                ))),
            ),
            None => (input, ParseResult::Suc(false)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::utils::parst::core::many1;

    use super::*;
    use quote::quote;

    #[test]
    fn test_get_ident() {
        let input = TokenIter::from(quote! {a});
        let (input, res) = getident().parse(input);
        if let ParseResult::Suc(i) = res {
            assert_eq!(i.to_string(), "a");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn group_ident() {
        let input = TokenIter::from(quote! {
            a { b c }
        });

        let parser = seq(
            matchident("a"),
            ingroup(Delimiter::Brace, seq(getident(), getident())),
        );

        if let (_, ParseResult::Suc((i, (j, k)))) = parser.parse(input) {
            assert_eq!(i, ());
            assert_eq!(j.to_string(), "b");
            assert_eq!(k.to_string(), "c");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn match_sep() {
        let input = TokenIter::from(quote! {
            a, b, c
        });

        let parser = many1(punctorend(','), getident());

        if let (_, ParseResult::Suc(c)) = parser.parse(input) {
            assert_eq!(c.len(), 3);
            assert_eq!(c[0].to_string(), "a");
            assert_eq!(c[1].to_string(), "b");
            assert_eq!(c[2].to_string(), "c");
        } else {
            assert!(false)
        }
    }
}
