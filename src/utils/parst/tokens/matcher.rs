use super::{SpannedCont, SpannedError, TokenIter};
use crate::utils::parst::core::{ParseResult, Parser};
use proc_macro2::{Spacing, TokenTree};

pub enum Matcher<'a> {
    Ident(&'a str),
    Punct(char, Spacing),
}

macro_rules! gen_match {
    ($($ts:tt)+) => {
        {
            use proc_macro2::{TokenTree};

            #[allow(unused_imports)]
            use proc_macro2::Spacing::*;
            use crate::utils::parst::tokens::Matcher::*;


            move |t: &Option<TokenTree>| match t {
                Some(TokenTree::Ident(i)) => {
                    let s = i.to_string();
                    matches!(Ident(&s), $($ts)+)
                },
                Some(TokenTree::Punct(p)) => {
                    matches!(Punct(p.as_char(), p.spacing()), $($ts)+)
                },
                // Literals and groups are unsupported
                _ => false,
            }
        }
    };
}
pub(crate) use gen_match;

/// parse a next token, based on a matcher, to produce and error.
pub struct Match<PG>
where
    PG: Fn(&Option<TokenTree>) -> bool,
{
    pub parser: PG,
    pub peek: bool,
}

impl<PG> Parser<TokenIter> for Match<PG>
where
    PG: Fn(&Option<TokenTree>) -> bool,
{
    type O = bool;
    type C = SpannedCont;
    type E = SpannedError;

    fn parse(&self, mut input: TokenIter) -> (TokenIter, ParseResult<Self::E, Self::C, Self::O>) {
        let res = if self.peek {
            (self.parser)(input.peek_next())
        } else {
            (self.parser)(&input.next())
        };
        (input, ParseResult::Suc(res))
    }
}

/// Matchers for basic true/false result parsers
/// - No error or continuation support, always succeed with true/false
/// - Used for easier predicates/choice parsers
///
/// ```ignore
/// // check if the next token is bob or jim
/// let parser1 = tkmatch!(peek => Ident("bob") | Ident("Jim"));
///
/// // consume the next token only if it is an identifier less than 10 characters
/// let parser2 = tkmatch!(cons => Ident(a) if a.len() < 10 );
///
/// // Get any punct that is alone
/// let parser3 = tkmatch!(cons => Punct(_, Alone));
/// ```
macro_rules! tkmatch {
    (peek => $($ts:tt)+) => {
        {
            use $crate::utils::parst::tokens::{gen_match, Match};
            Match { peek: true, parser: gen_match!($($ts)+) }
        }
    };
    (cons => $($ts:tt)+) => {
        {
            use $crate::utils::parst::tokens::{gen_match, Match};
            Match { peek: false, parser: gen_match!($($ts)+) }
        }
    }
}
pub(crate) use tkmatch;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::parst::{
        core::{seq, ParseResult},
        macros::seqs,
        tokens::{tkmatch, TokenIter},
    };
    use quote::quote;

    #[test]
    fn test_tkmatcher() {
        let q = TokenIter::from(quote! { a , bb ! += });

        let p = seqs!(
            tkmatch!(cons => Ident("a")),
            tkmatch!(cons => Punct(',', _)),
            tkmatch!(cons => Ident(x) if x.len() < 10),
            tkmatch!(cons => Punct('!', Alone)),
            tkmatch!(cons => Punct('+', Joint)),
            tkmatch!(cons => Punct('=', Alone))
        );

        if let (_, ParseResult::Suc(a)) = p.parse(q) {
            assert_eq!(a, (true, (true, (true, (true, (true, true))))));
        } else {
            assert!(false);
        }
    }
}
