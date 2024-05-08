use proc_macro2::TokenStream;
use quote::quote;

use crate::access::FieldState;

#[enumtrait::quick_enum]
#[enumtrait::store(index_kind_enum)]
pub enum IndexKind {
    Basic,
}

#[enumtrait::store(index_gen_trait)]
pub trait IndexGen {
    fn idx_t(&self) -> TokenStream;
    fn gen_state(&self) -> FieldState;
}

#[enumtrait::impl_trait(index_gen_trait for index_kind_enum)]
impl IndexGen for IndexKind {}

pub struct Basic;

impl IndexGen for Basic {
    fn idx_t(&self) -> TokenStream {
        quote! { ~Basic Index Type goes here~ }
    }

    fn gen_state(&self) -> FieldState {
        FieldState {
            datatype: quote! { () }.into(),
            init: quote! { () }.into(),
        }
    }
}
