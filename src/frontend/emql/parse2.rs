use proc_macro2::TokenStream;

use crate::{
    frontend::Diagnostics,
    utils::parst::{
        core::seq,
        tokens::{getident, matchident},
    },
};

use super::ast::AST;

pub(super) fn parse(ts: TokenStream, errs: &mut Diagnostics) -> Option<AST> {
    todo!()
}

#[cfg(test)]
mod test {
    use crate::utils::parst::{
        core::{seq, ParseResult, Parser},
        tokens::{getident, matchident, matchpunct, TokenIter},
    };
    use quote::quote;

    #[test]
    fn test_parser() {
        let input = TokenIter::from(quote! {
            name mydb;

            query name {};
        });

        let parser = seq(seq(matchident("name"), getident()), matchpunct(';'));

        if let (_, ParseResult::Suc(o)) = parser.parse(input) {
            println!("{:?}", o)
        }
    }
}
