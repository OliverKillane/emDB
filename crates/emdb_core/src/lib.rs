#![allow(dead_code)]
#![allow(unused_variables)]

extern crate proc_macro;

mod analysis;
mod optimise;
mod plan;
mod utils;
mod frontend;
mod backend;

mod macros {
    use proc_macro2::TokenStream;
    use quote::quote;
    use std::collections::LinkedList;

    pub(crate) fn make_impl<F: crate::frontend::Frontend>(tk: TokenStream) -> TokenStream {
        match F::from_tokens(tk) {
            Err(ds) => {
                for d in ds {
                    d.emit();
                }
                TokenStream::new()
            }
            Ok((lp, bks)) => {
                let mut errors = LinkedList::new();
                let impls = bks
                    .impls
                    .into_iter()
                    .filter_map(
                        |(id, backend)| match crate::backend::generate_code(backend, id, &lp) {
                            Ok(code) => Some(code),
                            Err(mut e) => {
                                errors.append(&mut e);
                                None
                            }
                        },
                    )
                    .collect::<Vec<_>>();
    
                for e in errors {
                    e.emit();
                }
    
                quote! {
                    #(#impls)*
                }
            }
        }
    }
}

#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn emql(tk: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crate::macros::make_impl::<crate::frontend::Emql>(tk.into()).into()
}
