//! Forwards rust expressions from the plan in order to check their code, when no backend impl is needed.
//! - Can be used for debugging.
//! - less costly, can run with no optimisers.
//! - useful for tests with no artifacts

// TODO; works best with arena mapping, develop this.


use crate::{
    analysis::interface::{self, types::translate_all_types}, plan, utils::misc::singlelist
};

use super::EMDBBackend;
mod impl_type;
use impl_query::translate_all_queries;
mod impl_query;
use impl_type::SemCheckTypes;

use proc_macro_error::{Diagnostic, Level};
use syn::spanned::Spanned;
use quote::quote;

// TODO:
// 1. Nice output to a file, with formatting
// 2. Expand the number of operators covered
// 3. set examples to use semcheck

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
        let types_preamble = translate_all_types(plan, &SemCheckTypes);
        let queries = translate_all_queries(plan);

        let tks = quote! {
            mod #impl_name { 
                #types_preamble 
                #queries
            }
        };
        println!("TOKENS: {tks}");

        Ok(tks)
    }
}
