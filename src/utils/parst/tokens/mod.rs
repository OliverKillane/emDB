//! [super::parst] implementation for parsing rust [Tokenstreams] with 1 token lookahead.

use super::core::{seq, ConComb, ErrComb, Parser, Recover};
use proc_macro2::{token_stream::IntoIter, Span, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, DiagnosticExt, SpanRange};
use std::collections::LinkedList;

mod basic;
mod derived;
mod error;
mod matcher;
mod recovery;

pub use basic::*;
pub use derived::*;
pub use error::*;
pub use matcher::*;
pub use recovery::*;

// TODO: make lookahead, lookback generic TokenIter<const Peek: usize, const Back: usize>
//       this requires some extra shenanigans and nightly to check const params, and const
//       evaluation on const params.

/// A wrapper for the tokentree iterator that allows for a 1-tokentree forward and back.
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

pub struct SpannedError {
    main: Diagnostic,
    prev: LinkedList<Diagnostic>,
}

impl SpannedError {
    fn from(main: Diagnostic) -> Self {
        Self {
            main,
            prev: LinkedList::new(),
        }
    }
}

pub struct SpannedCont(LinkedList<Diagnostic>);

impl SpannedCont {
    pub fn from_err(SpannedError { main, mut prev }: SpannedError) -> Self {
        prev.push_back(main);
        Self(prev)
    }

    pub fn into_list(self) -> LinkedList<Diagnostic> {
        self.0
    }
}

impl<O> ConComb<O, Self> for SpannedCont {
    fn combine_out(self, out: O) -> Self {
        // entirely ignore the output - we don't care about it
        self
    }

    fn combine_con(mut self, mut con: Self) -> Self {
        self.0.append(&mut con.0);
        self
    }
}

impl ErrComb<SpannedCont> for SpannedError {
    fn combine_con(self, mut con: SpannedCont) -> Self {
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
