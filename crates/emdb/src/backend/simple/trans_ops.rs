//! # Operation Translation
//! Converting operators to invocations of the [super::physical_ops]

use proc_macro2::TokenStream;

trait Translator {
    fn invoke(&self) -> TokenStream;
}