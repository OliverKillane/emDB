#![doc = include_str!("../README.md")]

use proc_macro2::TokenStream;
use quote::ToTokens;
use std::{any::type_name, marker::PhantomData};
use syn::parse2;

pub struct Tokens<T: syn::parse::Parse + ToTokens> {
    tks: TokenStream,
    phantom: PhantomData<T>,
}

impl<T: syn::parse::Parse + ToTokens> From<TokenStream> for Tokens<T> {
    fn from(value: TokenStream) -> Self {
        debug_assert!(
            parse2::<T>(value.clone()).is_ok(),
            "Tokensed a tokenstream of `{}` but could not parse it as such.",
            type_name::<T>()
        );
        Self {
            tks: value,
            phantom: PhantomData,
        }
    }
}

impl<T: syn::parse::Parse + ToTokens> ToTokens for Tokens<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tks.to_tokens(tokens)
    }
}
