use crate::core::{many0, maperr, or};

use super::super::core::{either, many1, mapsuc, recover, Parser};
use super::*;
use syn::parse::Parse as SynParse;

/// Use syn to parse tokens collected to a parser
pub fn syntopunct<
    T: SynParse,
    End: Parser<TokenIter, O = bool, C = SpannedCont, E = SpannedError>,
>(
    end: End,
) -> impl Parser<TokenIter, O = T, C = SpannedCont, E = SpannedError> {
    syn(collectuntil(or(end, isempty())))
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
            recover(item, recoveruptopunct(sep)),
        ),
    )
}

pub fn listseptrailing<I: Parser<TokenIter, C = SpannedCont, E = SpannedError>>(
    sep: char,
    item: I,
) -> impl Parser<TokenIter, O = Vec<I::O>, C = SpannedCont, E = SpannedError> {
    many0(
        not(isempty()),
        mapsuc(
            seq(
                recover(item, recoveruptopunct(sep)),
                either(isempty(), nothing(), mapsuc(matchpunct(sep), |_| ())),
            ),
            |(i, ())| i,
        ),
    )
}

pub fn not<I, P: Parser<I, O = bool>>(parser: P) -> impl Parser<I, O = bool, C = P::C, E = P::E> {
    mapsuc(parser, |t| !t)
}

pub fn embelisherr<P: Parser<TokenIter, C = SpannedCont, E = SpannedError>>(
    parser: P,
    msg: &'static str,
) -> impl Parser<TokenIter, O = P::O, C = SpannedCont, E = SpannedError> {
    maperr(parser, move |e| {
        SpannedError::from(e.main.note(String::from(msg)))
    })
}
