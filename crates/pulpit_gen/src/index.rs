use proc_macro2::TokenStream;
use quote::quote;

#[enumtrait::quick_enum]
#[enumtrait::store(index_kind_enum)]
pub enum Kind {
    Basic,
}

#[enumtrait::store(index_gen_trait)]
pub trait IndexGen {
    fn idx_t(&self) -> TokenStream;
    fn generate(&self) -> TokenStream;
}

#[enumtrait::impl_trait(index_gen_trait for index_kind_enum)]
impl IndexGen for Kind {}

pub struct Basic;

impl IndexGen for Basic {
    fn idx_t(&self) -> TokenStream {
        quote! { ~Basic Index Type goes here~ }
    }

    fn generate(&self) -> TokenStream {
        quote! { ~Basic Index Generation goes here~ }
    }
}
