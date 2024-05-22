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
        #[cfg(debug_assertions)]
        {
            if let Err(err) =  parse2::<T>(value.clone()) {
                panic!(
                    "Attempted to parse as `{}` but failed with message:\n`{}`\nTokens: `{}`",
                    type_name::<T>(),
                    err,
                    value
                )
            }
        }
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
