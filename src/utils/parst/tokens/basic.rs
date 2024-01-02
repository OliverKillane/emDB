use super::{
    super::core::{seq, ParseResult, Parser, Seq},
    *,
};
use proc_macro2::{Delimiter, Ident, Literal, Punct, Span, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};
use std::marker::PhantomData;
use syn::{parse::Parse as SynParse, spanned::Spanned};

pub struct GetIdent;
pub fn getident() -> GetIdent {
    GetIdent
}
impl Parser<TokenIter> for GetIdent {
    type O = Ident;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.next() {
            Some(TokenTree::Ident(i)) => (input, ParseResult::Suc(i)),
            Some(tt) => (
                input,
                ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected identifier, found {}", tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                        span,
                        proc_macro_error::Level::Error,
                        String::from("Expected identifier, found nothing"),
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
    type O = Ident;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.next() {
            Some(TokenTree::Ident(i)) => {
                if i == self.text {
                    (input, ParseResult::Suc(i))
                } else {
                    (
                        input,
                        ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                            i.span(),
                            proc_macro_error::Level::Error,
                            format!("Expected identifier {}, found {}", self.text, i),
                        ))),
                    )
                }
            }
            Some(tt) => (
                input,
                ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected identifier {}, found {}", self.text, tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                        span,
                        proc_macro_error::Level::Error,
                        format!("Expected identifier {}, found nothing", self.text),
                    ))),
                )
            }
        }
    }
}

pub struct PeekIdent {
    ident: &'static str,
}
pub fn peekident(ident: &'static str) -> PeekIdent {
    PeekIdent { ident }
}
impl Parser<TokenIter> for PeekIdent {
    type O = bool;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.peek_next() {
            Some(TokenTree::Ident(i)) if *i == self.ident => (input, ParseResult::Suc(true)),
            _ => (input, ParseResult::Suc(false)),
        }
    }
}

pub struct GetPunct;
pub fn getpunct() -> GetPunct {
    GetPunct
}
impl Parser<TokenIter> for GetPunct {
    type O = Punct;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.next() {
            Some(TokenTree::Punct(p)) => (input, ParseResult::Suc(p)),
            Some(tt) => (
                input,
                ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected punct, found {}", tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                        span,
                        proc_macro_error::Level::Error,
                        String::from("Expected punct, found nothing"),
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
    type O = Punct;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.next() {
            Some(TokenTree::Punct(p)) => {
                if p.as_char() == self.punct {
                    (input, ParseResult::Suc(p))
                } else {
                    (
                        input,
                        ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                            p.span(),
                            proc_macro_error::Level::Error,
                            format!("Expected punct {}, found {}", self.punct, p),
                        ))),
                    )
                }
            }
            Some(tt) => (
                input,
                ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected punct {}, found {}", self.punct, tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                        span,
                        proc_macro_error::Level::Error,
                        format!("Expected punct {}, found nothing", self.punct),
                    ))),
                )
            }
        }
    }
}

pub struct PeekPunct {
    punct: char,
}
pub fn peekpunct(punct: char) -> PeekPunct {
    PeekPunct { punct }
}
impl Parser<TokenIter> for PeekPunct {
    type O = bool;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.peek_next() {
            Some(TokenTree::Punct(p)) if p.as_char() == self.punct => {
                (input, ParseResult::Suc(true))
            }
            _ => (input, ParseResult::Suc(false)),
        }
    }
}

pub struct GetLiteral;
pub fn getliteral() -> GetLiteral {
    GetLiteral
}
impl Parser<TokenIter> for GetLiteral {
    type O = Literal;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        match input.next() {
            Some(TokenTree::Literal(l)) => (input, ParseResult::Suc(l)),
            Some(tt) => (
                input,
                ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected literal, found {}", tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                        span,
                        proc_macro_error::Level::Error,
                        String::from("Expected literal, found nothing"),
                    ))),
                )
            }
        }
    }
}

pub struct Terminal;
pub fn terminal() -> Terminal {
    Terminal
}
impl Parser<TokenIter> for Terminal {
    type O = ();
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        if let Some(tt) = input.next() {
            let big_span = input
                .extract_iter()
                .fold(tt.span(), |a, s| a.join(s.span()).unwrap());
            (
                // destroy the old iterator for a new one
                TokenIter::from(TokenStream::new()),
                ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                    big_span,
                    proc_macro_error::Level::Error,
                    String::from("Expected end of input"),
                ))),
            )
        } else {
            (input, ParseResult::Suc(()))
        }
    }
}

pub struct IsEmpty;
pub fn isempty() -> IsEmpty {
    IsEmpty
}
impl Parser<TokenIter> for IsEmpty {
    type O = bool;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        let res = input.peek_next().is_none();
        (input, ParseResult::Suc(res))
    }
}

pub struct InGroup<P: Parser<TokenIter, E = SpannedError, C = SpannedCont>> {
    delim: Delimiter,
    parser: Seq<TokenIter, SpannedError, SpannedCont, P, Terminal>,
}

pub fn ingroup<P: Parser<TokenIter, E = SpannedError, C = SpannedCont>>(
    delim: Delimiter,
    p: P,
) -> InGroup<P> {
    InGroup {
        delim,
        parser: seq(p, terminal()),
    }
}
impl<P: Parser<TokenIter, E = SpannedError, C = SpannedCont>> Parser<TokenIter> for InGroup<P> {
    type O = P::O;
    type C = P::C;
    type E = SpannedError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        fn delim_display(delim: Delimiter) -> &'static str {
            match delim {
                Delimiter::Parenthesis => "(...)",
                Delimiter::Brace => "{...}",
                Delimiter::Bracket => "[...]",
                Delimiter::None => "nothing",
            }
        }

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
                        ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                            g.span(),
                            proc_macro_error::Level::Error,
                            format!(
                                "Expected group with delimiter {:?}, found {:?}",
                                delim_display(self.delim),
                                delim_display(g.delimiter())
                            ),
                        ))),
                    )
                }
            }
            Some(tt) => (
                input,
                ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error::Level::Error,
                    format!("Expected group bracketed by {:?}", self.delim),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                        span,
                        proc_macro_error::Level::Error,
                        format!(
                            "Expected a following group bracketed by {} but found nothing",
                            delim_display(self.delim)
                        ),
                    ))),
                )
            }
        }
    }
}

/// Continually progresses the input until the parser succeeds, collects the tokens.
pub struct CollectUntil<P: Parser<TokenIter, E = SpannedError, C = SpannedCont, O = bool>> {
    parser: P,
}

pub fn collectuntil<P: Parser<TokenIter, E = SpannedError, C = SpannedCont, O = bool>>(
    parser: P,
) -> CollectUntil<P> {
    CollectUntil { parser }
}

impl<P: Parser<TokenIter, E = SpannedError, C = SpannedCont, O = bool>> Parser<TokenIter>
    for CollectUntil<P>
{
    type O = TokenStream;
    type C = SpannedCont;
    type E = SpannedError;
    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        let mut tks = vec![];
        loop {
            let (new_inp, res) = self.parser.parse(input);
            input = new_inp;
            match res {
                ParseResult::Suc(true) => {
                    break (input, ParseResult::Suc(TokenStream::from_iter(tks)))
                }
                ParseResult::Suc(false) => match input.next() {
                    Some(tk) => tks.push(tk),
                    None => {
                        let err = SpannedError::from(Diagnostic::spanned(
                            input.last_span().unwrap_or_else(Span::call_site),
                            proc_macro_error::Level::Error,
                            String::from("Unexpected end of input"),
                        ));
                        break (input, ParseResult::Err(err));
                    }
                },
                ParseResult::Con(c) => break (input, ParseResult::Con(c)),
                ParseResult::Err(e) => break (input, ParseResult::Err(e)),
            }
        }
    }
}

pub struct Syn<
    T: SynParse,
    P: Parser<TokenIter, O = TokenStream, C = SpannedCont, E = SpannedError>,
> {
    parser: P,
    _marker: PhantomData<T>,
}
pub fn syn<
    T: SynParse,
    P: Parser<TokenIter, O = TokenStream, C = SpannedCont, E = SpannedError>,
>(
    parser: P,
) -> Syn<T, P> {
    Syn {
        parser,
        _marker: PhantomData,
    }
}
impl<T: SynParse, P: Parser<TokenIter, O = TokenStream, C = SpannedCont, E = SpannedError>>
    Parser<TokenIter> for Syn<T, P>
{
    type O = T;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        let (new_inp, res) = self.parser.parse(input);
        match res {
            ParseResult::Suc(tks) => {
                if !tks.is_empty() {
                    match syn::parse2::<T>(tks.clone()) {
                        Err(es) => {
                            let spans = tks.span();
                            let err = Diagnostic::spanned(
                                spans,
                                Level::Error,
                                format!("Failed to parse {}", std::any::type_name::<T>()),
                            );

                            (
                                new_inp,
                                ParseResult::Err(SpannedError::from(
                                    err.span_error(es.span(), es.to_string()),
                                )),
                            )
                        }
                        Ok(o) => (new_inp, ParseResult::Suc(o)),
                    }
                } else {
                    let err = Diagnostic::spanned(
                        new_inp.last_span().unwrap_or_else(Span::call_site),
                        Level::Error,
                        format!("Expected {}, found nothing", std::any::type_name::<T>()),
                    );
                    (new_inp, ParseResult::Err(SpannedError::from(err)))
                }
            }
            ParseResult::Con(c) => (new_inp, ParseResult::Con(c)),
            ParseResult::Err(e) => (new_inp, ParseResult::Err(e)),
        }
    }
}

pub struct Nothing;
pub fn nothing() -> Nothing {
    Nothing
}
impl Parser<TokenIter> for Nothing {
    type C = SpannedCont;
    type E = SpannedError;
    type O = ();
    fn parse(&self, input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        (input, ParseResult::Suc(()))
    }
}

pub struct GetToken;
pub fn gettoken() -> GetToken {
    GetToken
}
impl Parser<TokenIter> for GetToken {
    type O = TokenTree;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        if let Some(tt) = input.next() {
            (input, ParseResult::Suc(tt))
        } else {
            let span = input.last_span().unwrap_or_else(Span::call_site);
            (
                input,
                ParseResult::Err(SpannedError::from(Diagnostic::spanned(
                    span,
                    proc_macro_error::Level::Error,
                    String::from("Expected token, found nothing"),
                ))),
            )
        }
    }
}
