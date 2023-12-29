use std::collections::LinkedList;

use proc_macro2::{Delimiter, TokenStream};
use proc_macro_error::Diagnostic;
use syn::token::Token;

use super::ast::AST;
use crate::utils::parst::{
    core::{
        either, many0, many1, map_suc, recover, seq, Either, MapSuc, ParseResult, Parser, Recover,
    },
    tokens::{
        getident, ingroup, matchident, matchpunct, peekident, peekpunct, recoverpunct, terminal,
        PeekIdent, SpannedCont, SpannedError, TokenIter,
    },
};

pub(super) fn parse(ts: TokenStream) -> Result<AST, LinkedList<Diagnostic>> {
    let parser = getident(); // todo

    let (_, res) = parser.parse(TokenIter::from(ts));

    match res {
        ParseResult::Suc(o) => Err(LinkedList::new()), // temporary
        ParseResult::Con(c) => Err(c.into_list()),
        ParseResult::Err(e) => Err(SpannedCont::from_err(e).into_list()),
    }
}

// fn name_parser() -> impl Parser<TokenIter> {
//     recover(
//         map_suc(
//             seq(matchident("name"), seq(getident(), matchpunct(';'))),
//             |(_, (name, _))| name,
//         ),
//         recoverpunct(';'),
//     )
// }

// fn param_list_parser() -> impl Parser<TokenIter> {

// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use quote::quote;

//     #[test]
//     fn test_name_parser() {
//         let ts = quote! {
//             name my_databaseaa;
//         };
//         let parser = name_parser();
//         let (out, res) = parser.parse(TokenIter::from(ts));
//         assert!(matches!(res, ParseResult::Suc(_)));
//     }
// }

// TODO:
// - add blob parser
// - add operator parser
// - fix punct recovery
// - implement parser below
// - sort out return types
// - redo examples

/*
let nameparser = map_suc(
    recover(
        seq(
            matchident("name"),
            seq(
                getident(),
                matchpunct(';')
            )
        ),
        recover_upto_punct(';'),
    ),
    |(_, (n, _))| n,
);

let argparser = map_suc(seq(
    getident(),
    seq(
        matchpunct(':'),
        syn_type(),
    )
), |_| SomethingType);


/// blob op blob op blob
/// blob := (name, operator(), let var, return), maybe op ;

// |> or ~> or <!
// |> value stream
// ~> ref stream
// <! mutate stream
let opparser = <custom>
;

// operators
sort((key, asc),*)
map(closure)
limit(i32)

let streamparser = recursive(|r|
   map_suc(
        blobparser(r),
        either(
            peek_punct(';'),
            matchpunct(';'),
            seq(
                operatorparser,
                r
            )
        ),
        |_| something
    )
);

let queryparser = map_suc(
    seq(
        matchident("query"),
        seq(
            getident(),
            seq(
                ingroup(Delimiter::Parentheses,
                    either(
                        is_empty(),
                        terminal(),
                        many0(
                            empty_or_peek_punct(','),
                            recover(
                                argparser,
                                recover_upto_punct(',')
                            )
                        )
                    )
                ),
                ingroup(
                    Delimiter::Braces,
                    either(
                        is_empty(),
                        terminal(),
                        many0(
                            empty_or_peek_punct(';'),
                            recover(
                                streamparser,
                                recover_upto_punct(';')
                            )
                        )
                    )
                )
            )
        )
    ),
    |_| Something::Query(...)
);

let tableparser = map_suc(, |_| Something::Table(...));

let query_or_table_parser = either(
    peek_ident("query"),
    queryparser,
    tableparser,
);

let parser = seq(
    nameparser,
    many0(
        empty_or_peek_punct(';'),
        recover(query_or_table_parser, recover_upto_punct(';'))
    )
);
*/
