use std::collections::LinkedList;

use proc_macro2::{Delimiter, Group, Ident, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};
use quote::{quote, ToTokens};
use syn::{parse2, ItemEnum, ItemImpl, ItemTrait};

fn enumtrait_internal_error(msg: &str) -> ! {
    panic!("Internal error in enumtrait: {}", msg)
}

pub fn extract_syn<T>(
    tks: TokenStream,
    f: impl Fn(TokenStream) -> syn::Result<T>,
) -> Result<T, LinkedList<Diagnostic>> {
    match f(tks) {
        Ok(o) => Ok(o),
        Err(errs) => Err(errs
            .into_iter()
            .map(|err| Diagnostic::spanned(err.span(), Level::Error, err.to_string()))
            .collect()),
    }
}

pub fn extract_group(tt: TokenTree, delim: &Delimiter) -> TokenStream {
    match tt {
        TokenTree::Group(g) if g.delimiter() == *delim => g.stream(),
        _ => enumtrait_internal_error("Expected a group with brace"),
    }
}

pub fn get_ident(tt: TokenTree) -> Ident {
    match tt {
        TokenTree::Ident(i) => i,
        _ => enumtrait_internal_error("Expected an identifier"),
    }
}

/// Manages serialization/deserialization when passing tokens (and their spans)
/// between proc macros.
pub trait CallStore: Sized {
    const DELIM: Delimiter = Delimiter::Brace;

    fn store(self) -> TokenStream;
    fn read(tks: TokenStream) -> Self;

    fn store_grouped(self) -> TokenStream {
        TokenTree::Group(Group::new(Self::DELIM, self.store())).into()
    }

    fn read_grouped(tk: TokenTree) -> Self {
        Self::read(extract_group(tk, &Self::DELIM))
    }
}

#[derive(Clone)]
pub struct ItemInfo<T: syn::parse::Parse + ToTokens + Clone>(pub T);
impl<T: syn::parse::Parse + ToTokens + Clone> CallStore for ItemInfo<T> {
    fn store(self) -> TokenStream {
        self.0.into_token_stream()
    }

    fn read(tks: TokenStream) -> Self {
        Self(extract_syn(tks, parse2::<T>).unwrap())
    }
}

pub struct InfoPair<A: CallStore, B: CallStore>(pub A, pub B);

impl<A: CallStore, B: CallStore> CallStore for InfoPair<A, B> {
    fn store(self) -> TokenStream {
        let Self(a, b) = self;
        let a_group = a.store_grouped();
        let b_group = b.store_grouped();
        quote! {
            #a_group
            #b_group
        }
    }

    fn read(tks: TokenStream) -> Self {
        let mut tks_iter = tks.into_iter();
        let data = Self(
            tks_iter.next().map(A::read_grouped).unwrap(),
            tks_iter.next().map(B::read_grouped).unwrap(),
        );
        assert!(tks_iter.next().is_none(), "Excess tokens in InfoPair");
        data
    }
}

pub struct IdentInfo(pub Ident);

impl CallStore for IdentInfo {
    fn store(self) -> TokenStream {
        self.0.into_token_stream()
    }

    fn read(tks: TokenStream) -> Self {
        let mut tks_iter = tks.into_iter();
        let data = Self(get_ident(tks_iter.next().unwrap()));
        assert!(tks_iter.next().is_none(), "Excess tokens in InfoIdent");
        data
    }
}
