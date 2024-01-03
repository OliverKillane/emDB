use crate::*;

use proc_macro2::{token_stream::IntoIter, Span, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, DiagnosticExt, SpanRange};
use std::collections::LinkedList;

pub mod basic;

/// A wrapper for [TokenStream] that allows for 1-token lookahead, and records the current and last [Span]s.
pub struct TokenIter {
    next: Option<TokenTree>,
    iter: IntoIter,
    curr_span: Option<Span>,
    prev_span: Option<Span>,
}

impl TokenIter {
    pub fn from(ts: TokenStream) -> Self {
        let mut iter = ts.into_iter();
        Self {
            next: iter.next(),
            iter,
            curr_span: None,
            prev_span: None,
        }
    }

    fn next(&mut self) -> Option<TokenTree> {
        let mut tk = self.iter.next();
        std::mem::swap(&mut self.next, &mut tk);
        self.prev_span = self.curr_span;
        self.curr_span = tk.as_ref().map(|t| t.span());
        tk
    }

    fn peek_next(&self) -> &Option<TokenTree> {
        &self.next
    }

    fn last_span(&self) -> &Option<Span> {
        &self.prev_span
    }

    fn extract_iter(self) -> IntoIter {
        self.iter
    }
}

/// Both an [error](Combi::Err) and [continuation](Combi::Con) that contains compiler diagnostcs.
pub struct TokenDiagnostic {
    main: Diagnostic,
    prev: LinkedList<Diagnostic>,
}

impl TokenDiagnostic {
    fn from(main: Diagnostic) -> Self {
        Self {
            main,
            prev: LinkedList::new(),
        }
    }

    fn combine(mut self, mut other: Self) -> Self {
        Self {
            main: self.main,
            prev: {
                self.prev.push_back(other.main);
                self.prev.append(&mut other.prev);
                self.prev
            },
        }
    }
}

impl CombiErr<TokenDiagnostic> for TokenDiagnostic {
    fn inherit_con(self, con: TokenDiagnostic) -> Self {
        self.combine(con)
    }

    fn catch_con(con: TokenDiagnostic) -> Self {
        con
    }
}

impl<S> CombiCon<S, TokenDiagnostic> for TokenDiagnostic {
    fn combine_suc(self, _: S) -> Self {
        self
    }

    fn combine_con(self, con: TokenDiagnostic) -> Self {
        self.combine(con)
    }
}

// NOTE: Potential rustc issue, when O is made a generic, we get an error as
//       TokenParser may be implemented for the combi::core types by a
//       downstream crate.
//       See: https://github.com/rust-lang/rust/issues/50237
pub trait TokenParser {
    type Out;

    fn parse(
        &self,
        input: TokenIter,
    ) -> (
        TokenIter,
        CombiResult<Self::Out, TokenDiagnostic, TokenDiagnostic>,
    );

    fn expected(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}

impl<P: TokenParser> Combi for P {
    type Suc = P::Out;
    type Err = TokenDiagnostic;
    type Con = TokenDiagnostic;
    type Inp = TokenIter;
    type Out = TokenIter;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        self.parse(input)
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.expected(f)
    }
}
