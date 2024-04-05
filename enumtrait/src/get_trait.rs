use std::collections::LinkedList;

use crate::passing::{
    extract_group, extract_syn, get_ident, CallStore, IdentInfo, InfoPair, ItemInfo,
};
use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};
use quote::{quote, ToTokens};
use syn::{parse2, spanned::Spanned, FnArg, ItemEnum, ItemTrait};

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

pub fn interface(
    attrs: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let (enum_macro, trait_macro) = parse_attrs(attrs)?;
    let info_tks = InfoPair(
        ItemInfo(extract_syn(item.clone(), parse2::<ItemTrait>)?),
        IdentInfo(trait_macro),
    )
    .store_grouped();

    Ok(quote! {
        #item
        #enum_macro!( enumtrait::get_trait_apply => #info_tks );
    })
}

fn parse_attrs(attrs: TokenStream) -> Result<(Ident, Ident), LinkedList<Diagnostic>> {
    let parser = mapsuc(
        seqs!(getident(), matchpunct('='), matchpunct('>'), getident()),
        |(enum_macro, (_, (_, trait_macro)))| (enum_macro, trait_macro),
    );

    let (_, res) = parser.comp(TokenIter::from(attrs, Span::call_site()));
    res.to_result().map_err(TokenDiagnostic::into_list)
}

pub fn apply(input: TokenStream) -> Result<TokenStream, Vec<Diagnostic>> {
    let InfoPair(InfoPair(ItemInfo(trait_info), IdentInfo(trait_macro)), ItemInfo(enum_info)) =
        InfoPair::read(input);

    check_trait(&trait_info)?;

    let pass_tks = InfoPair(
        ItemInfo::<ItemTrait>(trait_info),
        ItemInfo::<ItemEnum>(enum_info),
    )
    .store_grouped();

    Ok(quote! {
        macro_rules! #trait_macro {
            ($($t:tt)*) => {
                enumtrait::impl_trait_apply!( $($t)*  #pass_tks );
            }
        }
    })
}

fn check_trait(trait_def: &ItemTrait) -> Result<(), Vec<Diagnostic>> {
    fn unsupported(errors: &mut Vec<Diagnostic>, span: Span, kind: &str) {
        errors.push(Diagnostic::spanned(
            span,
            Level::Error,
            format!("{kind} are not supported by enumtrait"),
        ))
    }

    let mut errors = Vec::new();

    for item in &trait_def.items {
        match item {
            syn::TraitItem::Const(c) => unsupported(&mut errors, c.span(), "Constants"),
            syn::TraitItem::Type(t) => unsupported(&mut errors, t.span(), "Types"),
            syn::TraitItem::Macro(m) => unsupported(&mut errors, m.span(), "Macros"),
            syn::TraitItem::Verbatim(ts) => unsupported(&mut errors, ts.span(), "Arbitrary tokens"),
            syn::TraitItem::Fn(f) => {
                if !matches!(f.sig.inputs.first(), Some(&FnArg::Receiver(_))) {
                    errors.push(Diagnostic::spanned(f.sig.inputs.span(), Level::Error, "All trait functions need to start with a recieved (e.g. &self, &mut self, self)".to_owned()));
                }
            }
            _ => unsupported(&mut errors, Span::call_site(), "Unsupported trait item"),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
