#![warn(dead_code)]
#![warn(unused_variables)]
//! # Simple Serializable Backend
//! A basic backend producing code that uses [`pulpit`] generated tables.
//! - Allows for basic immutability optimisations, and append only tables.
//! - Generates a table object that uses parallelism internally, but only allows
//!   queries to execute in parallel if they are read only (normal borrow checker
//!   rules apply)

use combi::{
    core::{mapsuc, seq, setrepr},
    seqs,
    tokens::{
        basic::{collectuntil, isempty, matchident, matchpunct, syn},
        error::expectederr,
        TokenDiagnostic, TokenIter,
    },
    Combi,
};
use prettyplease::unparse;
use proc_macro2::TokenStream;
use proc_macro_error::{Diagnostic, Level};
use queries::QueriesInfo;
use quote::quote;
use std::{collections::LinkedList, fs::File, io::Write, path::Path};
use syn::{parse2, File as SynFile, LitStr};

use super::EMDBBackend;
use crate::utils::misc::singlelist;

mod closures;
mod namer;
mod operators;
mod queries;
mod tables;
mod types;

pub struct SimpleSerialized {
    debug: Option<LitStr>,
}

impl EMDBBackend for SimpleSerialized {
    const NAME: &'static str = "SimpleSerialized";

    fn parse_options(
        backend_name: &syn::Ident,
        options: Option<proc_macro2::TokenStream>,
    ) -> Result<Self, std::collections::LinkedList<proc_macro_error::Diagnostic>> {
        if let Some(opts) = options {
            let parser = expectederr(mapsuc(
                seqs!(
                    matchident("debug_file"),
                    matchpunct('='),
                    setrepr(syn(collectuntil(isempty())), "<file path>")
                ),
                |(_, (_, file))| Self { debug: Some(file) },
            ));
            let (_, res) = parser.comp(TokenIter::from(opts, backend_name.span()));
            res.to_result().map_err(TokenDiagnostic::into_list)
        } else {
            Ok(Self { debug: None })
        }
    }

    fn generate_code(
        self,
        impl_name: syn::Ident,
        plan: &crate::plan::Plan,
    ) -> Result<proc_macro2::TokenStream, std::collections::LinkedList<proc_macro_error::Diagnostic>>
    {
        let namer = namer::SimpleNamer::new();
        let tables::TableWindow {
            table_defs,
            datastore,
            datastore_impl,
            database,
            table_generated_info,
        } = tables::generate_tables(plan, &namer);

        let record_defs =
            types::generate_record_definitions(plan, &table_generated_info.get_types, &namer);

        let QueriesInfo {
            query_mod,
            query_impls,
        } = queries::generate_queries(plan, &table_generated_info, &namer);

        let namer::SimpleNamer { mod_tables, .. } = &namer;

        let tks = quote! {
            mod #impl_name {
                mod #mod_tables {
                    #(#table_defs)*
                }
                #query_mod
                #(#record_defs)*
                #datastore
                #datastore_impl
                #database
                #query_impls
            }
        };

        if let Some(debug_path) = self.debug {
            debug_output(&debug_path, tks.clone())?
        }

        Ok(tks)
    }
}

fn debug_output(debug_path: &LitStr, tks: TokenStream) -> Result<(), LinkedList<Diagnostic>> {
    match parse2::<SynFile>(tks) {
        Ok(m) => match File::create(Path::new(&debug_path.value())) {
            Ok(mut f) => match f.write_all(unparse(&m).as_bytes()) {
                Ok(()) => Ok(()),
                Err(e) => Err(singlelist(Diagnostic::spanned(
                    debug_path.span(),
                    Level::Error,
                    format!("Could not write to file: {e}"),
                ))),
            },
            Err(e) => Err(singlelist(Diagnostic::spanned(
                debug_path.span(),
                Level::Error,
                format!("Could not create file: {e}"),
            ))),
        },
        Err(e) => Err(singlelist(Diagnostic::spanned(
            debug_path.span(),
            Level::Error,
            format!("Could not parse code as file: {e}"),
        ))),
    }
}
