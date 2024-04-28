use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::super::Parseable;

#[derive(PartialEq, Eq, Debug)]
pub struct LongSequence {
    pub ids: Vec<Ident>,
}
impl Parseable for LongSequence {
    type Param = usize;

    fn generate_case(param: Self::Param) -> Self {
        let mut ids = Vec::new();
        for i in 0..param {
            ids.push(Ident::new(&format!("id{}", i), Span::call_site()));
        }
        LongSequence { ids }
    }

    fn generate_tokens(&self) -> TokenStream {
        let ids = &self.ids;
        quote! { #(#ids)* }
    }
}
