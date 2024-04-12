//! Forwards rust expressions from the plan in order to check their code, when no backend impl is needed.
//! - Can be used for debugging.
//! - less costly, can run with no optimisers.
//! - useful for tests with no artifacts

// TODO; works best with arena mapping, develop this.

mod types;
mod ops;

use crate::utils::misc::singlelist;

use super::EMDBBackend;
use proc_macro_error::{Diagnostic, Level};
use syn::spanned::Spanned;
use quote::quote;
pub struct SemCheck {}


impl EMDBBackend for SemCheck {
    const NAME: &'static str = "SemCheck";

    fn parse_options(
        backend_name: &syn::Ident,
        options: Option<proc_macro2::TokenStream>,
    ) -> Result<Self, std::collections::LinkedList<proc_macro_error::Diagnostic>> {
        if let Some(opts) = options {
            Err(singlelist(Diagnostic::spanned(
                opts.span(),
                Level::Error,
                "SemCheck backend does not take any options".to_owned(),
            )))
        } else {
            Ok(Self {})
        }
    }

    fn generate_code(
        self,
        impl_name: syn::Ident,
        plan: &crate::plan::Plan,
    ) -> Result<proc_macro2::TokenStream, std::collections::LinkedList<proc_macro_error::Diagnostic>> {
        Ok(quote! {
            mod #impl_name { }
        })
    }
}
