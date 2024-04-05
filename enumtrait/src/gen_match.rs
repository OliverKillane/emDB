use std::collections::LinkedList;

use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use syn::{
    parse2, punctuated::Punctuated, spanned::Spanned, Expr, Field, Fields, FieldsUnnamed, ItemEnum,
    Path, PathArguments, PathSegment, TypePath, Visibility,
};

use combi::{
    core::{mapsuc, nothing, seq},
    macros::seqs,
    tokens::{
        basic::{
            collectuntil, getident, isempty, matchident, matchpunct, recovgroup, syn, terminal,
        },
        TokenDiagnostic, TokenIter,
    },
    Combi, CombiResult,
};

use crate::passing::{extract_syn, CallStore, IdentInfo, InfoPair, ItemInfo};

pub fn interface(input: TokenStream) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let (enum_macro, inst_name, arg_ident, expression) = parse_input(input)?;

    let comm = InfoPair(
        ItemInfo(expression),
        InfoPair(IdentInfo(arg_ident), IdentInfo(inst_name)),
    )
    .store_grouped();

    Ok(quote! {
        #enum_macro !( enumtrait::gen_match => #comm)
    })
}

fn parse_input(input: TokenStream) -> Result<(Ident, Ident, Ident, Expr), LinkedList<Diagnostic>> {
    let parser = mapsuc(
        seqs!(
            getident(),
            matchident("as"),
            getident(),
            matchident("for"),
            getident(),
            matchpunct('-'),
            matchpunct('>'),
            syn(collectuntil(isempty()))
        ),
        |(enum_macro, (_, (inst_name, (_, (param, (_, (_, expression)))))))| {
            (enum_macro, inst_name, param, expression)
        },
    );

    let (_, res) = parser.comp(TokenIter::from(input, Span::call_site()));

    res.to_result().map_err(TokenDiagnostic::into_list)
}

pub fn apply(input: TokenStream) -> TokenStream {
    let InfoPair(
        InfoPair(ItemInfo(expression), InfoPair(IdentInfo(arg_ident), IdentInfo(inst_name))),
        ItemInfo(enum_item),
    ) = InfoPair::read(input);
    generate_match(enum_item, inst_name, arg_ident, expression)
}

fn generate_match(
    enum_item: ItemEnum,
    inst_name: Ident,
    arg_ident: Ident,
    expression: Expr,
) -> TokenStream {
    //INV: this enum was transformed by get_enum and has only `VariantName(Type)` members
    let exprs = enum_item
        .variants
        .into_iter()
        .map(|variant| {
            let cons = variant.ident;
            quote! {
                #cons(#arg_ident) => #expression,
            }
        })
        .collect::<Vec<_>>();

    quote!(
        match #inst_name {
            #(#exprs)*
        }
    )
}
