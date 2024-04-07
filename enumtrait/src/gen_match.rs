use std::collections::LinkedList;

use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::Diagnostic;
use quote::quote;
use syn::{Expr, ItemEnum};

use combi::{
    core::{mapsuc, seq},
    macros::seqs,
    tokens::{
        basic::{collectuntil, getident, isempty, matchident, matchpunct, syn},
        TokenDiagnostic, TokenIter,
        error::expectederr,
    },
    Combi,
};

use crate::macro_comm::{CallStore, IdentInfo, InfoPair, ItemInfo};

pub fn interface(input: TokenStream) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let (enum_macro, inst_name, arg_ident, expression) = parse_input(input)?;

    let comm = InfoPair(
        ItemInfo {
            data: expression,
            label: Ident::new("gen_match", Span::call_site()),
        },
        InfoPair(IdentInfo(arg_ident), IdentInfo(inst_name)),
    )
    .store_grouped();

    Ok(quote! {
        {
            use enumtrait::gen_match_apply;
            #enum_macro!( expr_ctx gen_match_apply => #comm )
        }
    })
}

fn parse_input(input: TokenStream) -> Result<(Ident, Ident, Ident, Expr), LinkedList<Diagnostic>> {
    let parser = expectederr(mapsuc(
        seqs!(
            getident(),
            matchident("as"),
            getident(),
            matchident("for"),
            getident(),
            matchpunct('='),
            matchpunct('>'),
            syn(collectuntil(isempty()))
        ),
        |(enum_macro, (_, (inst_name, (_, (param, (_, (_, expression)))))))| {
            (enum_macro, inst_name, param, expression)
        },
    ));
    let (_, res) = parser.comp(TokenIter::from(input, Span::call_site()));
    res.to_result().map_err(TokenDiagnostic::into_list)
}

pub fn apply(input: TokenStream) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let InfoPair(
        InfoPair(
            ItemInfo {
                data: expression,
                label: _,
            },
            InfoPair(IdentInfo(arg_ident), IdentInfo(inst_name)),
        ),
        ItemInfo {
            data: enum_item,
            label: _,
        },
    ) = InfoPair::read(input)?;
    Ok(generate_match(
        enum_item,
        &inst_name,
        &arg_ident,
        &expression,
    ))
}

fn generate_match(
    enum_item: ItemEnum,
    inst_name: &Ident,
    arg_ident: &Ident,
    expression: &Expr,
) -> TokenStream {
    let name = enum_item.ident;
    //INV: this enum was transformed by get_enum and has only `VariantName(Type)` members
    let exprs = enum_item
        .variants
        .into_iter()
        .map(|variant| {
            let cons = variant.ident;
            quote! {
                #name::#cons(#arg_ident) => #expression,
            }
        })
        .collect::<Vec<_>>();

    quote!(
        match #inst_name {
            #(#exprs)*
        }
    )
}
