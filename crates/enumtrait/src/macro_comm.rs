use std::collections::LinkedList;

use proc_macro2::{Delimiter, Group, Ident, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};
use quote::{quote, ToTokens};
use syn::parse2;

fn enumtrait_internal_error(msg: &str) -> ! {
    panic!("Internal error in enumtrait: {msg}")
}

pub fn extract_syn<T>(
    tks: TokenStream,
    label: &Ident,
    f: impl Fn(TokenStream) -> syn::Result<T>,
) -> Result<T, LinkedList<Diagnostic>> {
    match f(tks) {
        Ok(o) => Ok(o),
        Err(errs) => Err(errs
            .into_iter()
            .map(|err| {
                Diagnostic::spanned(
                    label.span(),
                    Level::Error,
                    format!("Error parsing tokens from store {label}"),
                )
                .span_error(err.span(), err.to_string())
            })
            .collect()),
    }
}

pub fn extract_group(tt: TokenTree, delim: Delimiter) -> TokenStream {
    match tt {
        TokenTree::Group(g) if g.delimiter() == delim => g.stream(),
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
    fn store_grouped(self) -> TokenStream {
        TokenTree::Group(Group::new(Self::DELIM, self.store())).into()
    }

    /// For reading we assume the structure is correct, but user provided items
    /// may be erroneous or invalid, so their errors are returned as diagnostics
    fn read(tks: TokenStream) -> Result<Self, LinkedList<Diagnostic>>;
    fn read_grouped(tk: TokenTree) -> Result<Self, LinkedList<Diagnostic>> {
        Self::read(extract_group(tk, Self::DELIM))
    }
}

/// Stores some tokens (that represent an item; enum, struct, trait, etc), with
/// the label they were stored in. This allows for parsing to reference the label.
#[derive(Clone)]
pub struct ItemInfo<T: syn::parse::Parse + ToTokens + Clone> {
    pub data: T,
    pub label: Ident,
}

impl<T: syn::parse::Parse + ToTokens + Clone> CallStore for ItemInfo<T> {
    fn store(self) -> TokenStream {
        let Self { data, label } = self;
        quote! { #label { #data } }
    }

    fn read(tks: TokenStream) -> Result<Self, LinkedList<Diagnostic>> {
        let mut tks_iter = tks.into_iter();
        let label = get_ident(tks_iter.next().unwrap());
        let data = extract_syn(
            extract_group(tks_iter.next().unwrap(), Self::DELIM),
            &label,
            parse2::<T>,
        )?;
        assert!(tks_iter.next().is_none(), "Excess tokens in ItemInfo");
        Ok(Self { data, label })
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

    fn read(tks: TokenStream) -> Result<Self, LinkedList<Diagnostic>> {
        let mut tks_iter = tks.into_iter();
        let data = Self(
            A::read_grouped(tks_iter.next().unwrap())?,
            B::read_grouped(tks_iter.next().unwrap())?,
        );
        assert!(tks_iter.next().is_none(), "Excess tokens in InfoPair");
        Ok(data)
    }
}

pub struct IdentInfo(pub Ident);

impl CallStore for IdentInfo {
    fn store(self) -> TokenStream {
        self.0.into_token_stream()
    }

    fn read(tks: TokenStream) -> Result<Self, LinkedList<Diagnostic>> {
        let mut tks_iter = tks.into_iter();
        let data = Self(get_ident(tks_iter.next().unwrap()));
        assert!(tks_iter.next().is_none(), "Excess tokens in InfoIdent");
        Ok(data)
    }
}

pub struct Triple<A: CallStore, B: CallStore, C: CallStore>(pub A, pub B, pub C);

impl<A: CallStore, B: CallStore, C: CallStore> CallStore for Triple<A, B, C> {
    fn store(self) -> TokenStream {
        let Self(a, b, c) = self;
        let a_group = a.store_grouped();
        let b_group = b.store_grouped();
        let c_group = c.store_grouped();
        quote! {
            #a_group
            #b_group
            #c_group
        }
    }

    fn read(tks: TokenStream) -> Result<Self, LinkedList<Diagnostic>> {
        let mut tks_iter = tks.into_iter();
        let data = Self(
            A::read_grouped(tks_iter.next().unwrap())?,
            B::read_grouped(tks_iter.next().unwrap())?,
            C::read_grouped(tks_iter.next().unwrap())?,
        );
        assert!(tks_iter.next().is_none(), "Excess tokens in InfoPair");
        Ok(data)
    }
}
