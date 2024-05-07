use super::ops;
use proc_macro2::TokenStream;
use quote::quote;

#[enumtrait::quick_enum]
#[enumtrait::store(kind_enum)]
pub enum Kind {
    DebugAccess,
}

struct AccessReq {
    prelude: TokenStream,
    access_methods: TokenStream,
}

#[enumtrait::store(access_gen_trait)]
pub trait AccessGen {
    fn state(&self) -> TokenStream;
    fn generate(&self) -> AccessReq;
    fn hook(&self, op: &ops::Kind) -> (TokenStream, TokenStream);
}

#[enumtrait::impl_trait(access_gen_trait for kind_enum)]
impl AccessGen for Kind {}

struct DebugAccess;

impl AccessGen for DebugAccess {
    fn state(&self) -> TokenStream {
        quote! {~DEBUG ACCESS STATE~}
    }
    fn generate(&self) -> AccessReq {
        AccessReq {
            prelude: quote! {~DEBUG ACCESS PRELUDE~},
            access_methods: quote! {~COOL METHOD~},
        }
    }

    fn hook(&self, op: &ops::Kind) -> (TokenStream, TokenStream) {
        (
            quote! {~DEBUG ACCESS BEFORE~},
            quote! {~DEBUG ACCESS AFTER~},
        )
    }
}
