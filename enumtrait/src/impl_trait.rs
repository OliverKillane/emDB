use proc_macro2::{Ident, Span, TokenStream, TokenTree, Group};
use proc_macro_error::{Diagnostic, Level};
use quote::{quote, ToTokens};
use syn::{
    parse2,
    ItemTrait,
    FnArg,
    spanned::Spanned,
};
use crate::passing::{extract_group, get_ident, extract_syn, CallStore};
use crate::get_enum::EnumInfo;


pub fn interface(attrs: TokenStream, item: TokenStream) -> Result<TokenStream, Vec<Diagnostic>> {
   
}

pub fn apply(input: TokenStream) -> Result<TokenStream, Vec<Diagnostic>> {
    println!("{input}");
    Ok(TokenStream::new())
}