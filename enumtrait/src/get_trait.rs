use proc_macro2::{Ident, Span, TokenStream, TokenTree, Group};
use proc_macro_error::{Diagnostic, Level};
use quote::{quote, ToTokens};
use syn::{
    parse2,
    ItemTrait,
    FnArg,
    spanned::Spanned,
};
use crate::passing::{extract_group, get_ident, extract_syn, CallStore, ItemInfo, TraitInfo, TraitEnumInfo};


pub fn interface(attrs: TokenStream, item: TokenStream) -> Result<TokenStream, Vec<Diagnostic>> {
    let (enum_macro, trait_macro) = parse_attrs(attrs)?;
    let trait_item = ItemInfo(extract_syn(item.clone(), parse2::<ItemTrait>)?);
    let info_tks = TraitInfo {
        trait_macro,
        trait_item,
    }.store_grouped();

    Ok(quote! {
        // keep the trait
        #item

        // call the previous macro (that already has the enum tokens) with the trait tokens
        #enum_macro!( #info_tks );
    })
}

fn parse_attrs(attrs: TokenStream) -> Result<(Ident, Ident), Vec<Diagnostic>> {
    let mut attrs_iter = attrs.into_iter();

    let enum_macro = match attrs_iter.next() {
        Some(TokenTree::Ident(i)) => Ok(i),
        Some(other) => Err(vec![Diagnostic::spanned(
            other.span(),
            Level::Error,
            "Expected the name of the output macro from enumtrait::get_enum".to_owned(),
        ).help("#[enumtrait::get_enum(my_macro)], then use in #[enumtrait::get_trait(my_macro -> my_impl_macro)]".to_owned())]),
        None => Err(vec![Diagnostic::spanned(
            Span::call_site(),
            Level::Error,
            "Expected the name of the output macro from enumtrait::get_enum".to_owned(),
        ).help("#[enumtrait::get_enum(my_macro)], then use in #[enumtrait::get_trait(my_macro -> my_impl_macro)]".to_owned())]),
    }?;

    match (attrs_iter.next(), attrs_iter.next()) {
        (Some(TokenTree::Punct(p1)), Some(TokenTree::Punct(p2)))
            if p1.as_char() == '=' && p2.as_char() == '>' =>
        {
            Ok(())
        }
        (Some(p1), _) => Err(vec![Diagnostic::spanned(
            p1.span(),
            Level::Error,
            "Expected '->'".to_owned(),
        )
        .help("#[enumtrait::get_trait(my_enum_macro -> my_trait_macro)]".to_owned())]),
        _ => Err(vec![Diagnostic::spanned(
            Span::call_site(),
            Level::Error,
            "Expected '->'".to_owned(),
        )
        .help(
            "#[enumtrait::get_trait(my_enum_macro -> my_trait_macro)]".to_owned(),
        )]),
    }?;

    let next_macro = match attrs_iter.next() {
        Some(TokenTree::Ident(i)) => Ok(i),
        Some(other) => Err(vec![Diagnostic::spanned(
            other.span(),
            Level::Error,
            "Expected the name of the output macro from enumtrait::get_trait".to_owned(),
        ).help("#[enumtrait::get_trait(my_macro -> my_impl_macro)] can be used in #[enumtrait::impl_trait(my_impl_macro)]".to_owned())]),
        None => Err(vec![Diagnostic::spanned(
            Span::call_site(),
            Level::Error,
            "Expected the name of the output macro from enumtrait::get_enum".to_owned(),
        ).help("#[enumtrait::get_trait(my_macro -> my_impl_macro)] can be used in #[enumtrait::impl_trait(my_impl_macro)]".to_owned())]),
    }?;

    if attrs_iter.next().is_some() {
        Err(vec![Diagnostic::spanned(
            Span::call_site(),
            Level::Error,
            "No extra arguments should be provided".to_owned(),
        )])
    } else {
        Ok((enum_macro, next_macro))
    }
}

pub fn apply(input: TokenStream) -> Result<TokenStream, Vec<Diagnostic>> {
    let TraitEnumInfo { trait_info, enum_info }  = TraitEnumInfo::read(input);
    check_trait(&trait_info.trait_item.0)?;
    let macro_name = trait_info.trait_macro.clone();

    Ok(quote! {
        macro_rules! #macro_name {
            ($($t:tt)*) => {
                enumtrait::get_trait_apply!( $($t)*  #pass_tks );
            }
        }
    })    
}

fn check_trait(trait_def: &ItemTrait) -> Result<(), Vec<Diagnostic>> {
    fn unsupported(errors: &mut Vec<Diagnostic>, span: Span, kind: &str) {
        errors.push(Diagnostic::spanned(span, Level::Error, format!("{kind} are not supported by enumtrait")))
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
            },
            _ => unsupported(&mut errors, Span::call_site(), "Unsupported trait item"),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
