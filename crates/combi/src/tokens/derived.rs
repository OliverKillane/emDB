use syn::parse::Parse as SynParse;

use crate::{
    core::{choice, mapsuc, nothing, recover, seq},
    derived::{many0, many1},
    logical::{not, or},
};

use super::{
    basic::{collectuntil, isempty, matchpunct, peekpunct, syn},
    recovery::until,
    TokenParser,
};

/// Use syn to parse tokens collected to a parser
#[inline]
pub fn syntopunct<T: SynParse, End: TokenParser<bool>>(end: End) -> impl TokenParser<T> {
    syn(collectuntil(or(end, isempty())))
}

/// Parse a punct separated list
#[inline]
pub fn listsep<S, I: TokenParser<S> + Clone>(sep: char, item: I) -> impl TokenParser<Vec<S>> {
    choice(
        isempty(),
        mapsuc(nothing(), |()| vec![]),
        many1(
            choice(
                peekpunct(sep),
                mapsuc(matchpunct(sep), |_| true),
                mapsuc(nothing(), |_| false),
            ),
            recover(item, until(peekpunct(sep))),
        ),
    )
}

#[inline]
pub fn listseptrailing<S, I: TokenParser<S>>(sep: char, item: I) -> impl TokenParser<Vec<S>> {
    many0(
        not(isempty()),
        mapsuc(
            seq(
                recover(item, until(peekpunct(sep))),
                choice(isempty(), nothing(), mapsuc(matchpunct(sep), |_| ())),
            ),
            |(i, ())| i,
        ),
    )
}
