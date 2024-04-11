use proc_macro2::TokenStream;

use super::super::Parseable;

#[derive(PartialEq, Eq, Debug)]
pub struct Nothing;

impl Parseable for Nothing {
    type Param = ();

    fn generate_case((): Self::Param) -> Self {
        Nothing
    }

    fn generate_tokens(&self) -> TokenStream {
        TokenStream::new()
    }
}
