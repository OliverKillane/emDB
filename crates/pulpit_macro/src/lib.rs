use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;
use pulpit_gen::selector::{MutabilitySelector, SelectorImpl};
use quote::ToTokens;

#[proc_macro_error]
#[proc_macro]
pub fn simple(tokens: TokenStream) -> TokenStream {
    match pulpit_gen::macros::simple::simple(tokens.into()) {
        Ok(ts) => MutabilitySelector
            .select_table(ts)
            .generate(&pulpit_gen::namer::CodeNamer::pulpit(), vec![])
            .into_token_stream()
            .into(),
        Err(es) => {
            for e in es {
                e.emit();
            }
            TokenStream::new()
        }
    }
}
