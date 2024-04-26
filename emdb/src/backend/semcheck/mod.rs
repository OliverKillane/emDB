//! Forwards rust expressions from the plan in order to check their code, when no backend impl is needed.
//! - Can be used for debugging.
//! - less costly, can run with no optimisers.
//! - useful for tests with no artifacts

// TODO; works best with arena mapping, develop this.


use std::{collections::LinkedList, fs::File, io::Write, path::Path};

use crate::{
    analysis::interface::{self, types::translate_all_types}, plan, utils::misc::singlelist
};

use super::EMDBBackend;
mod impl_type;
use combi::{core::{mapsuc, seq, setrepr}, seqs, tokens::{basic::{collectuntil, isempty, matchident, matchpunct, peekpunct, syn}, error::expectederr, TokenDiagnostic, TokenIter}, Combi};
use impl_query::translate_all_queries;
mod impl_query;
use impl_type::SemCheckTypes;

use proc_macro2::TokenStream;
use proc_macro_error::{Diagnostic, Level};
use syn::{parse2, spanned::Spanned, File as SynFile, ItemMod, LitStr};
use quote::quote;
use prettyplease::unparse;

// TODO:
// 1. Nice output to a file, with formatting
// 2. Expand the number of operators covered
// 3. set examples to use semcheck

pub struct SemCheck {
    debug: Option<LitStr>
}

fn debug_output(debug_path: &LitStr, tks: TokenStream) -> Result<(), LinkedList<Diagnostic>> {
    match parse2::<SynFile>(tks) {
        Ok(m) => {
            match File::create(Path::new(&debug_path.value())) {
                Ok(mut f) => {
                    match f.write(unparse(&m).as_bytes()) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(singlelist(Diagnostic::spanned(debug_path.span(), Level::Error, format!("Could not write to file: {e}")))),
                    }
                },
                Err(e) => Err(singlelist(Diagnostic::spanned(debug_path.span(), Level::Error, format!("Could not create file: {e}"))))
            }
        },
        Err(e) => Err(singlelist(Diagnostic::spanned(debug_path.span(), Level::Error, format!("Could not parse code as file: {e}")))),
    }
}

impl EMDBBackend for SemCheck {
    const NAME: &'static str = "SemCheck";

    fn parse_options(
        backend_name: &syn::Ident,
        options: Option<proc_macro2::TokenStream>,
    ) -> Result<Self, std::collections::LinkedList<proc_macro_error::Diagnostic>> {
        if let Some(opts) = options {
            let parser = expectederr(mapsuc(seqs!(
                matchident("debug_file"),
                matchpunct('='),
                setrepr(syn(collectuntil(isempty())), "<file path>")
            ), |(_, (_, file))| Self { debug: Some(file)}));
            let (_, res) = parser.comp(TokenIter::from(opts, backend_name.span()));
            res.to_result().map_err(TokenDiagnostic::into_list)
        } else {
            Ok(Self { debug: None})
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

        if let Some(debug_path) = self.debug {
            debug_output(&debug_path, tks.clone())?
        }

        Ok(tks)
    }
}
