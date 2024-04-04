use proc_macro2::{Delimiter, Group, Ident, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};
use syn::{ parse2, ItemEnum, ItemImpl, ItemTrait};
use quote::{quote, ToTokens};

fn enumtrait_internal_error(msg: &str) -> ! {
    panic!("Internal error in enumtrait: {}", msg)
}

pub fn extract_syn<T>(
    tks: TokenStream,
    f: impl Fn(TokenStream) -> syn::Result<T>,
) -> Result<T, Vec<Diagnostic>> {
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
impl <T: syn::parse::Parse + ToTokens + Clone> CallStore for ItemInfo<T> {
    fn store(self) -> TokenStream {
        self.0.into_token_stream()
    }

    fn read(tks: TokenStream) -> Self {
        Self(extract_syn(tks, parse2::<T>).unwrap())
    }
}

pub struct TraitInfo {
    pub trait_macro: Ident,
    pub trait_item: ItemInfo<ItemTrait>,
}

impl CallStore for TraitInfo {
    fn store(self) -> TokenStream {
       let Self { trait_macro, trait_item } = self;
        let trait_group =  trait_item.store_grouped();
        quote!{
            #trait_macro #trait_group
        }
    }

    fn read(tks: TokenStream) -> Self {
        let mut tokens = tks.into_iter();
        let trait_macro_raw = tokens.next().unwrap();
        let trait_item_raw = tokens.next().unwrap();
        assert!(tokens.next().is_none());

        Self { 
            trait_macro: get_ident(trait_macro_raw), 
            trait_item:  ItemInfo::read_grouped(trait_item_raw),
        }
    }
}

pub struct TraitEnumInfo {
    pub trait_info: TraitInfo,
    pub enum_info: ItemInfo<ItemEnum>,
}

impl CallStore for TraitEnumInfo {
    fn store(self) -> TokenStream {
        let Self { trait_info, enum_info } = self;
        let trait_group = trait_info.store_grouped();
        let enum_group = enum_info.store_grouped();
        quote!{
            #trait_group
            #enum_group
        }
    }

    fn read(tks: TokenStream) -> Self {
        let mut tks_iter = tks.into_iter();
        Self { 
            trait_info: tks_iter.next().map(TraitInfo::read_grouped).unwrap(), 
            enum_info: tks_iter.next().map(ItemInfo::read_grouped).unwrap()
        }
    }
}

pub struct ImplInfo {
    pub impl_info: ItemInfo<ItemImpl>,
    pub trait_info: ItemInfo<ItemTrait>,
    pub enum_info: ItemInfo<ItemEnum>,
}

impl CallStore for ImplInfo {
    fn store(self) -> TokenStream {
        let Self { impl_info, trait_info, enum_info } = self;
        let impl_group = impl_info.store_grouped();
        let trait_group = trait_info.store_grouped();
        let enum_group = enum_info.store_grouped();
        quote!{
            #impl_group
            #trait_group
            #enum_group
        }
    }

    fn read(tks: TokenStream) -> Self {
        let mut tks_iter = tks.into_iter();
        Self { 
            impl_info: tks_iter.next().map(ItemInfo::read_grouped).unwrap(), 
            trait_info: tks_iter.next().map(ItemInfo::read_grouped).unwrap(), 
            enum_info: tks_iter.next().map(ItemInfo::read_grouped).unwrap()
        }
    }
}