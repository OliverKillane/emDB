use super::super::core::{either, many1, map_suc, recover, Parser, Recover};
use super::*;
use syn::parse::Parse as SynParse;

/// Use syn to parse tokens collected to a parser
fn syntopunct<T: SynParse>(
    punct: char,
) -> impl Parser<TokenIter, O = T, C = SpannedCont, E = SpannedError> {
    syn::<T, CollectUntil<PeekPunct>>(collectuntil(peekpunct(punct)))
}

/// Parse a punct separated list
fn listsep<
    I: Parser<TokenIter, C = SpannedCont, E = SpannedError>,
    R: Recover<TokenIter, SpannedError, C = SpannedCont>,
>(
    sep: char,
    item: I,
    endrecover: R,
) -> impl Parser<TokenIter, C = SpannedCont, E = SpannedError> {
    recover(
        either(
            isempty(),
            map_suc(nothing(), |()| vec![]),
            many1(
                either(
                    peekpunct(sep),
                    map_suc(matchpunct(sep), |_| true),
                    map_suc(nothing(), |_| false),
                ),
                recover(item, recoverpunct(sep)),
            ),
        ),
        endrecover,
    )
}
