use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::ToTokens;

#[proc_macro_error]
#[proc_macro]
pub fn simple(tokens: TokenStream) -> TokenStream {
    match pulpit_gen::macros::simple::simple(tokens.into()) {
        Ok(ts) => pulpit_gen::selector::select_basic(ts).generate(&pulpit_gen::namer::CodeNamer::pulpit())
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
