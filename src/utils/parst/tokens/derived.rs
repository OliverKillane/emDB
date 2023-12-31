use super::super::core::{either, many1, mapsuc, recover, Parser, Recover};
use super::*;
use syn::parse::Parse as SynParse;

/// Use syn to parse tokens collected to a parser
pub fn syntopunct<T: SynParse>(
    punct: char,
) -> impl Parser<TokenIter, O = T, C = SpannedCont, E = SpannedError> {
    syn::<T, CollectUntil<PeekPunct>>(collectuntil(peekpunct(punct)))
}

/// Parse a punct separated list
pub fn listsep<I: Parser<TokenIter, C = SpannedCont, E = SpannedError>>(
    sep: char,
    item: I,
) -> impl Parser<TokenIter, O = Vec<I::O>, C = SpannedCont, E = SpannedError> {
    either(
        isempty(),
        mapsuc(nothing(), |()| vec![]),
        many1(
            either(
                peekpunct(sep),
                mapsuc(matchpunct(sep), |_| true),
                mapsuc(nothing(), |_| false),
            ),
            recover(item, recoverpunct(sep)),
        ),
    )
}

pub fn not<I, P: Parser<I, O = bool>>(parser: P) -> impl Parser<I, O = bool, C = P::C, E = P::E> {
    mapsuc(parser, |t| !t)
}
