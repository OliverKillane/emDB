use ints::IntPrototype;
use proc_macro::TokenStream;
use proc_macro_error2::{Diagnostic, proc_macro_error};
use proc_macro2::Span;

mod ints;

#[proc_macro_error]
#[proc_macro]
pub fn generate_int(tokens: TokenStream) -> TokenStream {
    if !tokens.is_empty() {
        Diagnostic::spanned(
            Span::call_site(),
            proc_macro_error2::Level::Error,
            String::from(
                "The `generate` macro does not accept any arguments. 
It is used to generate the code for the `arbints` crate.",
            ),
        )
        .emit();
        return TokenStream::new();
    } else {
        IntPrototype::all()
            .map(|p| p.generate())
            .map(TokenStream::from)
            .collect()
    }
}

mod test;