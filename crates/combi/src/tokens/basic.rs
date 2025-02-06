//! Basic token combinators.
//! - Literals are not included as they are more complex to parse, consider using [`struct@syn::LitStr`] instead.
use std::marker::PhantomData;

use super::{TokenDiagnostic, TokenIter};
use crate::{
    core::{seqdiff, DiffRes},
    Combi, CombiErr, CombiResult, Repr,
};
use derive_where::derive_where;
use proc_macro2::{Delimiter, Ident, Literal, Punct, Span, TokenStream, TokenTree};
use proc_macro_error2::{Diagnostic, Level};
use syn::{parse::Parse as SynParse, spanned::Spanned};

#[derive(Clone, Debug)]
pub struct GetIdent;
pub fn getident() -> GetIdent {
    GetIdent
}
impl Combi for GetIdent {
    type Suc = Ident;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        mut input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        match input.next() {
            Some(TokenTree::Ident(i)) => (input, CombiResult::Suc(i)),
            Some(tt) => (
                input,
                CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error2::Level::Error,
                    format!("Expected identifier, found {}", tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                        span,
                        proc_macro_error2::Level::Error,
                        String::from("Expected identifier, found nothing"),
                    ))),
                )
            }
        }
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "<ident>")
    }
}

#[derive(Clone, Debug)]
pub struct MatchIdent {
    text: &'static str,
}
pub fn matchident(text: &'static str) -> MatchIdent {
    MatchIdent { text }
}
impl Combi for MatchIdent {
    type Suc = Ident;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        mut input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        match input.next() {
            Some(TokenTree::Ident(i)) => {
                if i == self.text {
                    (input, CombiResult::Suc(i))
                } else {
                    (
                        input,
                        CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                            i.span(),
                            proc_macro_error2::Level::Error,
                            format!("Expected identifier {}, found {}", self.text, i),
                        ))),
                    )
                }
            }
            Some(tt) => (
                input,
                CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error2::Level::Error,
                    format!("Expected identifier {}, found {}", self.text, tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                        span,
                        proc_macro_error2::Level::Error,
                        format!("Expected identifier {}, found nothing", self.text),
                    ))),
                )
            }
        }
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.text)
    }
}

#[derive(Debug, Clone)]
pub struct PeekIdent {
    ident: &'static str,
}
pub fn peekident(ident: &'static str) -> PeekIdent {
    PeekIdent { ident }
}
impl Combi for PeekIdent {
    type Suc = bool;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        let res = CombiResult::Suc(
            matches!(input.peek_next(), Some(TokenTree::Ident(i)) if *i == self.ident),
        );
        (input, res)
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.ident)
    }
}

#[derive(Debug, Clone)]
pub struct GetPunct;
pub fn getpunct() -> GetPunct {
    GetPunct
}

impl Combi for GetPunct {
    type Suc = Punct;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        mut input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        match input.next() {
            Some(TokenTree::Punct(p)) => (input, CombiResult::Suc(p)),
            Some(tt) => (
                input,
                CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error2::Level::Error,
                    format!("Expected punct, found {}", tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                        span,
                        proc_macro_error2::Level::Error,
                        String::from("Expected punct, found nothing"),
                    ))),
                )
            }
        }
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "<punct>")
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct matchpunct(pub char);
impl Combi for matchpunct {
    type Suc = Punct;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        mut input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        match input.next() {
            Some(TokenTree::Punct(p)) => {
                if p.as_char() == self.0 {
                    (input, CombiResult::Suc(p))
                } else {
                    (
                        input,
                        CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                            p.span(),
                            proc_macro_error2::Level::Error,
                            format!("Expected punct {}, found {}", self.0, p),
                        ))),
                    )
                }
            }
            Some(tt) => (
                input,
                CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error2::Level::Error,
                    format!("Expected punct {}, found {}", self.0, tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                        span,
                        proc_macro_error2::Level::Error,
                        format!("Expected punct {}, found nothing", self.0),
                    ))),
                )
            }
        }
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct PeekPunct {
    punct: char,
}
pub fn peekpunct(punct: char) -> PeekPunct {
    PeekPunct { punct }
}
impl Combi for PeekPunct {
    type Suc = bool;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        let res = CombiResult::Suc(
            matches!(input.peek_next(), Some(TokenTree::Punct(p)) if p.as_char() == self.punct),
        );
        (input, res)
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.punct)
    }
}

#[derive(Clone, Debug)]
pub struct GetLiteral;
pub fn getliteral() -> GetLiteral {
    GetLiteral
}
impl Combi for GetLiteral {
    type Suc = Literal;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        mut input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        match input.next() {
            Some(TokenTree::Literal(l)) => (input, CombiResult::Suc(l)),
            Some(tt) => (
                input,
                CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                    tt.span(),
                    proc_macro_error2::Level::Error,
                    format!("Expected literal, found {}", tt),
                ))),
            ),
            None => {
                let span = input.last_span().unwrap_or_else(Span::call_site);
                (
                    input,
                    CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                        span,
                        proc_macro_error2::Level::Error,
                        String::from("Expected literal, found nothing"),
                    ))),
                )
            }
        }
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "<literal>")
    }
}

#[derive(Clone, Debug)]
pub struct IsEmpty;
pub fn isempty() -> IsEmpty {
    IsEmpty
}
impl Combi for IsEmpty {
    type Suc = bool;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        let res = input.peek_next().is_none();
        (input, CombiResult::Suc(res))
    }

    fn repr(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

fn delim_sep(delim: Delimiter) -> (&'static str, &'static str) {
    match delim {
        Delimiter::Parenthesis => ("(", ")"),
        Delimiter::Brace => ("{", "}"),
        Delimiter::Bracket => ("[", "]"),
        Delimiter::None => ("nothing", "nothing"),
    }
}

fn delim_display(delim: Delimiter) -> String {
    let (l, r) = delim_sep(delim);
    format!("{}...{}", l, r)
}

fn describe_tokentree(tt: &TokenTree) -> String {
    match tt {
        TokenTree::Ident(i) => i.to_string(),
        TokenTree::Punct(p) => p.to_string(),
        TokenTree::Literal(l) => l.to_string(),
        TokenTree::Group(g) => delim_display(g.delimiter()),
    }
}

/// Runs a parser on a bracketed group.
/// - Recovers from errors inside the group
/// - Recovers from incorrect bracket type usage
/// - The parser must consume all input
// TODO: look into exploiting paralellism here for `seq(recovgroup(...), ...)`
pub fn recovgroup<P>(delim: Delimiter, parser: P) -> RecovGroup<P>
where
    P: Combi<Inp = TokenIter, Out = TokenIter, Con = TokenDiagnostic, Err = TokenDiagnostic>,
{
    RecovGroup(delim, seqdiff(parser, terminal))
}

#[derive(Clone, Debug)]
pub struct RecovGroup<P>(pub Delimiter, pub seqdiff<P, terminal>)
where
    P: Combi<Inp = TokenIter, Out = TokenIter, Con = TokenDiagnostic, Err = TokenDiagnostic>;

impl<P> Combi for RecovGroup<P>
where
    P: Combi<Inp = TokenIter, Out = TokenIter, Con = TokenDiagnostic, Err = TokenDiagnostic>,
{
    type Suc = P::Suc;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        mut input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        let map_res = |res| match res {
            CombiResult::Suc((s, ())) => CombiResult::Suc(s),
            CombiResult::Con(c) => CombiResult::Con(c),
            CombiResult::Err(e) => CombiResult::Err(e),
        };

        match input.next() {
            Some(TokenTree::Group(g)) => {
                let (new_inp, res) = self
                    .1
                    .comp(TokenIter::from(g.stream(), g.delim_span().open()));

                // INV: if the combi is successful, then we know terminal was successful, and hence the
                //      output must be `DiffRes::Second(())`
                assert!(if matches!(res, CombiResult::Suc(_)) {
                    matches!(new_inp, DiffRes::Second(()))
                } else {
                    true
                });

                let mapped_res = map_res(res);

                if g.delimiter() != self.0 {
                    // we allow continuation past incorrect brackets
                    let con = TokenDiagnostic::from(Diagnostic::spanned(
                        g.delim_span().join(),
                        Level::Error,
                        format!(
                            "Expected {} group, found {} group",
                            delim_display(self.0),
                            delim_display(g.delimiter())
                        ),
                    ));

                    (
                        input,
                        match mapped_res {
                            CombiResult::Suc(_) => CombiResult::Con(con),
                            CombiResult::Con(c) => CombiResult::Con(c.inherit_con(con)),
                            CombiResult::Err(e) => CombiResult::Con(e.inherit_con(con)),
                        },
                    )
                } else {
                    (input, mapped_res)
                }
            }
            Some(tt) => (
                input,
                CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                    tt.span(),
                    Level::Error,
                    format!(
                        "Expected {} group, found {}",
                        delim_display(self.0),
                        describe_tokentree(&tt)
                    ),
                ))),
            ),
            None => {
                let span = *input.cur_span();
                (
                    input,
                    CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                        span,
                        Level::Error,
                        format!("Expected {} group, found nothing", delim_display(self.0)),
                    ))),
                )
            }
        }
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let (l, r) = delim_sep(self.0);
        write!(f, "{l}{}{r}", Repr(&self.1))
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct collectuntil<P>(pub P)
where
    P: Combi<
        Inp = TokenIter,
        Out = TokenIter,
        Suc = bool,
        Err = TokenDiagnostic,
        Con = TokenDiagnostic,
    >;

impl<P> Combi for collectuntil<P>
where
    P: Combi<
        Inp = TokenIter,
        Out = TokenIter,
        Suc = bool,
        Err = TokenDiagnostic,
        Con = TokenDiagnostic,
    >,
{
    type Suc = TokenStream;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        mut input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        let mut tks = Vec::new();
        loop {
            let (new_inp, res) = self.0.comp(input);
            input = new_inp;
            match res {
                CombiResult::Suc(true) => {
                    break (input, CombiResult::Suc(TokenStream::from_iter(tks)))
                }
                CombiResult::Suc(false) => match input.next() {
                    Some(tk) => tks.push(tk),
                    None => {
                        let err = TokenDiagnostic::from(Diagnostic::spanned(
                            input.last_span().unwrap_or_else(Span::call_site),
                            proc_macro_error2::Level::Error,
                            String::from("Unexpected end of input"),
                        ));
                        break (input, CombiResult::Err(err));
                    }
                },
                CombiResult::Con(c) => break (input, CombiResult::Con(c)),
                CombiResult::Err(e) => break (input, CombiResult::Err(e)),
            }
        }
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.repr(f)
    }
}

pub fn syn<T, P>(p: P) -> Syn<T, P>
where
    T: SynParse,
    P: Combi<
        Inp = TokenIter,
        Out = TokenIter,
        Suc = TokenStream,
        Con = TokenDiagnostic,
        Err = TokenDiagnostic,
    >,
{
    Syn {
        parser: p,
        _marker: PhantomData,
    }
}

#[derive_where(Clone; P)]
#[derive_where(Debug; P)]
pub struct Syn<T, P>
where
    T: SynParse,
    P: Combi<
        Inp = TokenIter,
        Out = TokenIter,
        Suc = TokenStream,
        Con = TokenDiagnostic,
        Err = TokenDiagnostic,
    >,
{
    parser: P,
    _marker: PhantomData<T>,
}

impl<T, P> Combi for Syn<T, P>
where
    T: SynParse,
    P: Combi<
        Inp = TokenIter,
        Out = TokenIter,
        Suc = TokenStream,
        Con = TokenDiagnostic,
        Err = TokenDiagnostic,
    >,
{
    type Suc = T;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        let (new_inp, res) = self.parser.comp(input);
        match res {
            CombiResult::Suc(tks) => {
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
                                CombiResult::Err(TokenDiagnostic::from(
                                    err.span_error(es.span(), es.to_string()),
                                )),
                            )
                        }
                        Ok(o) => (new_inp, CombiResult::Suc(o)),
                    }
                } else {
                    let err = Diagnostic::spanned(
                        new_inp.last_span().unwrap_or_else(Span::call_site),
                        Level::Error,
                        format!("Expected {}, found nothing", std::any::type_name::<T>()),
                    );
                    (new_inp, CombiResult::Err(TokenDiagnostic::from(err)))
                }
            }
            CombiResult::Con(c) => (new_inp, CombiResult::Con(c)),
            CombiResult::Err(e) => (new_inp, CombiResult::Err(e)),
        }
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "<{}>", std::any::type_name::<T>())
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct gettoken;

impl Combi for gettoken {
    type Suc = TokenTree;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    #[inline]
    fn comp(
        &self,
        mut input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Suc, TokenDiagnostic, TokenDiagnostic>,
    ) {
        if let Some(tt) = input.next() {
            (input, CombiResult::Suc(tt))
        } else {
            let span = *input.cur_span();
            (
                input,
                CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                    span,
                    proc_macro_error2::Level::Error,
                    String::from("Expected token, found nothing"),
                ))),
            )
        }
    }

    fn repr(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct terminal;

impl Combi for terminal {
    type Suc = ();
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = ();

    #[inline]
    fn comp(
        &self,
        mut input: Self::Inp,
    ) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        if let Some(tt) = input.next() {
            // NOTE: `a.join` returns None on Stable, and always Some on nightly.
            let big_span = if cfg!(feature = "nightly") {
                // INV: On nightly the result of the join is always Some(..)
                #[allow(clippy::unwrap_used)]
                input
                    .extract_iter()
                    .fold(tt.span(), |a, s| a.join(s.span()).unwrap())
            } else {
                TokenStream::from_iter(input.iter).span()
            };
            (
                (),
                CombiResult::Err(TokenDiagnostic::from(Diagnostic::spanned(
                    big_span,
                    proc_macro_error2::Level::Error,
                    String::from("Expected end of input"),
                ))),
            )
        } else {
            ((), CombiResult::Suc(()))
        }
    }

    fn repr(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}
