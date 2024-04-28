//! Parser combinators for consuming [tokenstreams](TokenStream) to create [rust compiler diagnostics](Diagnostic).
//! - No backtracking is permitted
//! - 1-token of lookahead provided by [TokenIter::peek_next]

use proc_macro2::{token_stream::IntoIter, Span, TokenStream, TokenTree};
use proc_macro_error::Diagnostic;
use std::collections::LinkedList;

use crate::{Combi, CombiCon, CombiErr};

pub mod basic;
pub mod derived;
pub mod error;
pub mod matcher;
pub mod recovery;

/// A wrapper for [TokenStream] that allows for 1-token lookahead, and records the current and last [Span]s.
pub struct TokenIter {
    next: Option<TokenTree>,
    iter: IntoIter,
    curr_span: Span,
    prev_span: Option<Span>,
}

impl TokenIter {
    pub fn from(ts: TokenStream, start_span: Span) -> Self {
        let mut iter = ts.into_iter();
        Self {
            next: iter.next(),
            iter,
            curr_span: start_span,
            prev_span: Some(start_span),
        }
    }

    /// Advance to the next token, and return `Some(token)` if present, otherwise return `None` and do not advance.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<TokenTree> {
        if let Some(ref tk) = self.next {
            self.prev_span = Some(self.curr_span);
            self.curr_span = tk.span();
            let mut some_tk = self.iter.next();
            std::mem::swap(&mut self.next, &mut some_tk);
            some_tk
        } else {
            None
        }
    }

    pub fn peek_next(&self) -> &Option<TokenTree> {
        &self.next
    }

    /// The span of the last token from [Self::next()].
    pub fn cur_span(&self) -> &Span {
        &self.curr_span
    }

    /// The span of the last, last token found from [Self::next()].
    pub fn last_span(&self) -> &Option<Span> {
        &self.prev_span
    }

    pub fn extract_iter(self) -> IntoIter {
        self.iter
    }
}

/// Both an [error](Combi::Err) and [continuation](Combi::Con) that contains compiler diagnostcs.
#[derive(Debug)]
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

    pub fn into_list(self) -> LinkedList<Diagnostic> {
        let mut list = self.prev;
        list.push_back(self.main);
        list
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
// NOTE: Ideally I would make an alias, that binds the associated types of a
//       Combi impl, then implement that, however we run into significant issues
//       with potentially conflicting traits
// TODO: create a binding for recovery and parsers, like with:
//       ```rust
//       trait Alias: Trait<Item=char> {}
//       impl<T: Trait<Item=char>> Alias for T {}
//       ```

/// An alias for the [Combi] trait, for token parsers
pub trait TokenParser<S>:
    Combi<Suc = S, Err = TokenDiagnostic, Con = TokenDiagnostic, Inp = TokenIter, Out = TokenIter>
{
}
impl<
        S,
        T: Combi<
            Suc = S,
            Err = TokenDiagnostic,
            Con = TokenDiagnostic,
            Inp = TokenIter,
            Out = TokenIter,
        >,
    > TokenParser<S> for T
{
}

/// An alias for the [Combi] trait, for recovery parsers
pub trait TokenRecover<S>:
    Combi<
    Suc = S,
    Err = TokenDiagnostic,
    Con = TokenDiagnostic,
    Inp = (TokenDiagnostic, TokenIter),
    Out = TokenIter,
>
{
}
impl<
        S,
        T: Combi<
            Suc = S,
            Err = TokenDiagnostic,
            Con = TokenDiagnostic,
            Inp = (TokenDiagnostic, TokenIter),
            Out = TokenIter,
        >,
    > TokenRecover<S> for T
{
}
