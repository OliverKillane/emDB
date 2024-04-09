use combi::{
    core::{choice, mapsuc, nothing, recursive, seq},
    derived::many0,
    logical::not,
    macros::choices,
    tokens::{
        basic::{getident, isempty, matchpunct, peekpunct, recovgroup},
        TokenIter, TokenParser,
    },
};
use proc_macro2::{Delimiter, Span, TokenStream};

use super::super::{LongSequence, Nothing, Parse, RecursiveIdent};

pub struct CombiParser;

fn quick_parse<T>(parser: impl TokenParser<T>, input: TokenStream) -> T {
    parser
        .comp(TokenIter::from(input, Span::call_site()))
        .1
        .to_result()
        .unwrap()
}

impl Parse<RecursiveIdent> for CombiParser {
    fn parse(input: TokenStream) -> RecursiveIdent {
        let parser = recursive(|h| {
            choices! {
                peekpunct('!') => mapsuc(matchpunct('!'), |_| RecursiveIdent::Final),
                otherwise => mapsuc(
                    seq(getident(), recovgroup(Delimiter::Brace, h.clone())),
                    |(id, recur)| RecursiveIdent::Next { id: id.to_string(), recur: Box::new(recur) })
            }
        });
        quick_parse(parser, input)
    }
}

impl Parse<LongSequence> for CombiParser {
    fn parse(input: TokenStream) -> LongSequence {
        let parser = mapsuc(many0(not(isempty()), getident()), |ids| LongSequence {
            ids,
        });
        quick_parse(parser, input)
    }
}

impl Parse<Nothing> for CombiParser {
    fn parse(input: TokenStream) -> Nothing {
        let parser = mapsuc(nothing(), |()| Nothing);
        quick_parse(parser, input)
    }
}
