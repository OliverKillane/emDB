use super::super::{LongSequence, Nothing, Parse, RecursiveIdent, LargeGroups};
use chumsky::prelude::*;
use chumsky_proc::prelude::*;
use proc_macro2::{Delimiter, Punct};
pub struct ChumskyProc;

fn recur_ident_parser(
) -> impl Parser<RustToken, Box<RecursiveIdent>, Error = Simple<RustToken, RustSpan>> {
    recursive(|r| {
        filter_map(RustToken::filter_ident)
            .then(r.delimited_by(
                just(RustToken::StartDelim(Delimiter::Brace)),
                just(RustToken::EndDelim(Delimiter::Brace)),
            ))
            .map(|(id, recur)| {
                Box::new(RecursiveIdent::Next {
                    id: id.to_string(),
                    recur: recur,
                })
            })
            .or(punct('!').map(|_| Box::new(RecursiveIdent::Final)))
    })
}

impl Parse<RecursiveIdent> for ChumskyProc {
    fn parse(input: proc_macro2::TokenStream) -> RecursiveIdent {
        let parser = recur_ident_parser();
        *(parser.parse(stream_from_tokens(input.into()))).unwrap()
    }
}

fn long_parser() -> impl Parser<RustToken, LongSequence, Error = Simple<RustToken, RustSpan>> {
    filter_map(RustToken::filter_ident)
        .repeated()
        .collect::<Vec<_>>()
        .map(|ids| LongSequence { ids })
}

impl Parse<LongSequence> for ChumskyProc {
    fn parse(input: proc_macro2::TokenStream) -> LongSequence {
        long_parser()
            .parse(stream_from_tokens(input.into()))
            .unwrap()
    }
}

fn nothing_parser() -> impl Parser<RustToken, Nothing, Error = Simple<RustToken, RustSpan>> {
    end().map(|_| Nothing)
}

impl Parse<Nothing> for ChumskyProc {
    fn parse(input: proc_macro2::TokenStream) -> Nothing {
        nothing_parser()
            .parse(stream_from_tokens(input.into()))
            .unwrap()
    }
}

fn large_group_parser() -> impl Parser<RustToken, LargeGroups, Error = Simple<RustToken, RustSpan>> {
    recursive(|r| 
        filter_map(RustToken::filter_punct).repeated().at_least(1).map(|ps| LargeGroups::Puncts(ps.iter().map(Punct::as_char).collect())).delimited_by(
            just(RustToken::StartDelim(Delimiter::Parenthesis)),
            just(RustToken::EndDelim(Delimiter::Parenthesis))
        ).or(
            r.delimited_by(
                just(RustToken::StartDelim(Delimiter::Brace)), 
                just(RustToken::EndDelim(Delimiter::Brace))
            )
        ).repeated().at_least(1).map(|gs| if gs.len() == 1 { gs[0].clone()} else {LargeGroups::Groups(gs)})
    )
}

impl Parse<LargeGroups> for ChumskyProc {
    fn parse(input: proc_macro2::TokenStream) -> LargeGroups {
        large_group_parser()
            .parse(stream_from_tokens(input.into()))
            .unwrap()
    }
}