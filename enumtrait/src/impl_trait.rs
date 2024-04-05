use std::collections::{HashMap, LinkedList};

use crate::passing::{extract_group, extract_syn, get_ident, CallStore, InfoPair, ItemInfo};
use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};
use quote::{quote, ToTokens};
use syn::{
    parse2, spanned::Spanned, FnArg, GenericArgument, GenericParam, Generics, ImplItem, ImplItemFn,
    ItemEnum, ItemImpl, ItemTrait, TraitItem, TraitItemFn, TypeParam,
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

pub fn interface(
    attrs: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let macro_name = parse_attrs(attrs)?;
    let trait_item = ItemInfo(extract_syn(item.clone(), parse2::<ItemImpl>)?).store_grouped();

    Ok(quote! {
        #item
        #macro_name!( #trait_item );
    })
}

fn parse_attrs(attrs: TokenStream) -> Result<Ident, LinkedList<Diagnostic>> {
    let (_, res) = getident().comp(TokenIter::from(attrs, Span::call_site()));
    res.to_result().map_err(TokenDiagnostic::into_list)
}

pub fn apply(input: TokenStream) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let InfoPair(ItemInfo(impl_item), InfoPair(ItemInfo(trait_item), ItemInfo(enum_item))) =
        InfoPair::read(input);

    Ok(generate_impl(impl_item, trait_item, enum_item)?.into_token_stream())
}

fn generate_impl(
    mut impl_item: ItemImpl,
    trait_item: ItemTrait,
    enum_item: ItemEnum,
) -> Result<ItemImpl, LinkedList<Diagnostic>> {
    // we should let the user define other stuff
    if impl_item.items.is_empty() {
        let x = &impl_item;
        let y = &trait_item;

        println!("{x:#?} \n\n\n {y:#?}");

        for method in trait_item.items {
            // todo
        }

        Ok(impl_item)
    } else {
        let mut errs = LinkedList::new();
        errs.push_back(Diagnostic::spanned(
            impl_item.span(),
            Level::Error,
            "Expected an empty impl block".to_owned(),
        ));
        Err(errs)
    }
}

// fn extract_impl_generics(impl_item: &ItemImpl) -> Result<impl Iterator<Item=&GenericArgument>, Vec<Diagnostic>> {
//     todo!()
// }

// fn get_generic_mapping(trait_generics: &Generics, params: impl Iterator<Item=&GenericArgument>) -> Result<HashMap<&GenericParam, &GenericParam>, Vec<Diagnostic>> {
//     todo!()
// }

// fn generate_method() {

// }
