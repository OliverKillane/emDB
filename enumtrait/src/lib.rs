use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::TokenStream;
use proc_macro_error::{proc_macro_error, Diagnostic};

mod gen_match;
mod get_enum;
mod get_trait;
mod impl_trait;
mod passing;

fn emit_result<ERR: IntoIterator<Item = Diagnostic>>(
    res: Result<TokenStream, ERR>,
) -> CompilerTokenStream {
    match res {
        Ok(out) => out,
        Err(es) => {
            for e in es {
                e.emit();
            }
            TokenStream::new()
        }
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn get_enum(attrs: CompilerTokenStream, item: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(get_enum::interface(attrs.into(), item.into()))
}

#[proc_macro_error]
#[proc_macro]
pub fn gen_match(input: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(gen_match::interface(input.into()))
}

#[proc_macro]
pub fn gen_match_apply(input: CompilerTokenStream) -> CompilerTokenStream {
    gen_match::apply(input.into()).into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn get_trait(attrs: CompilerTokenStream, item: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(get_trait::interface(attrs.into(), item.into()))
}

#[proc_macro_error]
#[proc_macro]
pub fn get_trait_apply(input: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(get_trait::apply(input.into()))
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn impl_trait(attrs: CompilerTokenStream, item: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(impl_trait::interface(attrs.into(), item.into()))
}

#[proc_macro_error]
#[proc_macro]
pub fn impl_trait_apply(input: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(impl_trait::apply(input.into()))
}
