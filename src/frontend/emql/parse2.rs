use std::collections::LinkedList;

use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;

use crate::utils::parst::{
    core::{recover, seq, ParseResult, Parser, Recover},
    tokens::{getident, matchident, matchpunct, recoverpunct, SpannedCont, TokenIter},
};

use super::ast::AST;

pub(super) fn parse(ts: TokenStream) -> Result<AST, LinkedList<Diagnostic>> {
    let name_parser = recover(
        seq(seq(matchident("name"), getident()), matchpunct(';')),
        recoverpunct(';'),
    );

    let parser = name_parser;

    let (_, res) = parser.parse(TokenIter::from(ts));
    match res {
        ParseResult::Suc(o) => Err(LinkedList::new()), // temporary
        ParseResult::Con(c) => Err(c.to_list()),
        ParseResult::Err(e) => Err(SpannedCont::from_err(e).to_list()),
    }
}
