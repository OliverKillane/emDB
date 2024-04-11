use combi::{
    core::{choice, mapsuc, nothing, recursive, seq},
    derived::many0,
    logical::not,
    macros::choices,
    tokens::{
        basic::{getident, getpunct, isempty, matchpunct, peekpunct, recovgroup}, matcher::matcher, TokenIter, TokenParser
    },
};
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};

use crate::cases::large_groups::LargeGroups;

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

impl Parse<LargeGroups> for CombiParser {
    fn parse(input: TokenStream) -> LargeGroups {
        let parser = recursive(|r| choice(
            matcher::<true, _>(|tk| matches!(tk, TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis), ""),
            mapsuc(recovgroup(Delimiter::Parenthesis, many0(not(isempty()), getpunct())), |ps| LargeGroups::Puncts(ps.iter().map(|p| p.as_char()).collect())),
            mapsuc(recovgroup(Delimiter::Brace, many0(not(isempty()), r.clone())), |gs| LargeGroups::Groups(gs)),
        ));
        quick_parse(parser, input)
    }
}