// TODO: Sort
// // trait Matcher

// use std::marker::PhantomData;

// use proc_macro2::{Ident, Spacing, Span, TokenTree};

// use crate::{Combi, CombiResult};

// use super::{TokenDiagnostic, TokenIter, TokenParser};

// #[derive(Clone)]
// struct Matcher<const PEEK: bool, F, I>
// where
//     F: Fn(&I) -> bool,
// {
//     match_fn: F,
//     repr: &'static str,
//     peek: bool,
//     _marker: PhantomData<I>,
// }

// pub enum MatcherAny<'a> {
//     Ident(&'a str),
//     Punct(char, Spacing),
// }

// impl<F> TokenParser for Matcher<true, F, Ident>
// where
//     F: Fn(&Ident) -> bool + Clone,
// {
//     type Out = Ident;

//     fn parse(
//         &self,
//         input: TokenIter,
//     ) -> (
//         TokenIter,
//         CombiResult<Self::Out, TokenDiagnostic, TokenDiagnostic>,
//     ) {

//     }

//     fn expected(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         write!(f, "{}", self.repr)
//     }
// }

// // give me the punct / ident
// // macro_rules! matchident {}
// // macro_rules! matchpunct {}
// // macro_rules! matchany {}

// // boolean
// // macro_rules! peekident {}
// // macro_rules! peekpunct {}
// // macro_rules! peekany {}
