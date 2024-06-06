#![warn(dead_code)]
#![warn(unused_variables)]
//! # Simple Serializable Backend
//! A basic backend producing code that uses [`pulpit`] generated tables.
//! - Allows for basic immutability optimisations, and append only tables.
//! - Generates a table object that uses parallelism internally, but only allows
//!   queries to execute in parallel if they are read only (normal borrow checker
//!   rules apply)

use combi::{
    core::mapsuc,
    tokens::{
        basic::{
            collectuntil, getident, isempty, syn,
        },
        options::{OptEnd, OptField, OptParse},
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

use super::{interface::InterfaceTrait, EMDBBackend};
use crate::utils::misc::singlelist;

mod closures;
pub mod namer;
mod operators;
mod queries;
mod tables;
mod types;

pub struct Serialized {
    debug: Option<LitStr>,
    interface: Option<InterfaceTrait>,
}

impl EMDBBackend for Serialized {
    const NAME: &'static str = "Serialized";

    fn parse_options(
        backend_name: &syn::Ident,
        options: Option<proc_macro2::TokenStream>,
    ) -> Result<Self, std::collections::LinkedList<proc_macro_error::Diagnostic>> {
        if let Some(opts) = options {
            let parser = (
                OptField::new("debug_file", ||syn(collectuntil(isempty()))),
                (OptField::new("interface", ||mapsuc(getident(), |name| InterfaceTrait{ name })), OptEnd),
            )
                .gen('=');
            let (_, res) = parser.comp(TokenIter::from(opts, backend_name.span()));
            res.to_result()
                .map_err(TokenDiagnostic::into_list)
                .map(|(debug, (interface, ()))| Serialized { debug, interface })
        } else {
            Ok(Self {
                debug: None,
                interface: None,
            })
        }
    }

    fn generate_code(
        self,
        impl_name: syn::Ident,
        plan: &crate::plan::Plan,
    ) -> Result<proc_macro2::TokenStream, std::collections::LinkedList<proc_macro_error::Diagnostic>>
    {
        let namer = namer::SerializedNamer::new();
        let tables::TableWindow {
            table_defs,
            datastore,
            datastore_impl,
            database,
            table_generated_info,
        } = tables::generate_tables(plan, &self.interface, &namer);

        let record_defs =
            types::generate_record_definitions(plan, &table_generated_info.get_types, &namer);

        let QueriesInfo {
            query_mod,
            query_impls,
        } = queries::generate_queries(plan, &table_generated_info, &self.interface, &namer);

        let namer::SerializedNamer { mod_tables, .. } = &namer;

        let tks = quote! {
            mod #impl_name {
                #![allow(non_shorthand_field_patterns)] // current name field printing is `fielname: fieldname`
                use emdb::dependencies::minister::Physical; //TODO: remove and use better operator selection
                pub mod #mod_tables {
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
