#![warn(dead_code)]
#![warn(unused_variables)]
//! # Simple Serializable Backend
//! A basic backend producing code that uses [`pulpit`] generated tables.
//! - Allows for basic immutability optimisations, and append only tables.
//! - Generates a table object that uses parallelism internally, but only allows
//!   queries to execute in parallel if they are read only (normal borrow checker
//!   rules apply)

use combi::{
    core::{choice, mapsuc},
    macros::choices,
    tokens::{
        basic::{collectuntil, getident, gettoken, isempty, matchident, peekident, syn},
        error::error,
        options::{OptEnd, OptField, OptParse},
        TokenDiagnostic, TokenIter, TokenParser,
    },
    Combi,
};
use prettyplease::unparse;
use proc_macro2::TokenStream;
use proc_macro_error2::{Diagnostic, Level};
use pulpit::gen::selector::{
    ColumnarSelector, CopySelector, MutabilitySelector, TableSelectors, ThunderdomeSelector,
};
use queries::QueriesInfo;
use quote::quote;
use std::{collections::LinkedList, fs::File, io::Write, path::Path};
use syn::{parse2, File as SynFile, Ident, LitStr};

use super::{interface::InterfaceTrait, EMDBBackend};
use crate::utils::{misc::singlelist, on_off::on_off};
use operators::OperatorImpls;

mod closures;
pub mod namer;
mod operators;
mod queries;
mod tables;
mod types;
mod stats;

pub struct Serialized {
    debug: Option<LitStr>,
    interface: Option<InterfaceTrait>,
    public: bool,
    ds_name: Option<Ident>,
    aggressive_inlining: bool,
    operator_impl: OperatorImpls,
    table_selector: TableSelectors,
}

fn operator_impl_parse() -> impl TokenParser<OperatorImpls> {
    choices! (
        peekident("Basic") => mapsuc(matchident("Basic"), |_| OperatorImpls::Basic),
        peekident("Iter") => mapsuc(matchident("Iter"), |_| OperatorImpls::Iter),
        peekident("Parallel") => mapsuc(matchident("Parallel"), |_| OperatorImpls::Parallel),
        peekident("Chunk") => mapsuc(matchident("Chunk"), |_| OperatorImpls::Chunk),
        otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, "Invalid Operator Choice".to_owned()))
    )
}

fn table_select_parse() -> impl TokenParser<TableSelectors> {
    choices! (
        peekident("Mutability") => mapsuc(matchident("Mutability"), |_| MutabilitySelector.into()),
        peekident("Thunderdome") => mapsuc(matchident("Thunderdome"), |_| ThunderdomeSelector.into()),
        peekident("Columnar") => mapsuc(matchident("Columnar"), |_| ColumnarSelector.into()),
        peekident("Copy") => mapsuc(matchident("Copy"), |_| CopySelector.into()),
        otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, "Invalid Table Selector Choice".to_owned()))
    )
}

impl EMDBBackend for Serialized {
    const NAME: &'static str = "Serialized";

    fn parse_options(
        backend_name: &syn::Ident,
        options: Option<proc_macro2::TokenStream>,
    ) -> Result<Self, std::collections::LinkedList<proc_macro_error2::Diagnostic>> {
        const DEFAULT_OP_IMPL: OperatorImpls = OperatorImpls::Parallel;
        const DEFAULT_TABLE_SELECTOR: TableSelectors =
            TableSelectors::MutabilitySelector(MutabilitySelector);
        if let Some(opts) = options {
            let parser = (
                OptField::new("debug_file", || syn(collectuntil(isempty()))),
                (
                    OptField::new("interface", || {
                        mapsuc(getident(), |name| InterfaceTrait { name })
                    }),
                    (
                        OptField::new("pub", on_off),
                        (
                            OptField::new("ds_name", getident),
                            (
                                OptField::new("aggressive_inlining", on_off),
                                (
                                    OptField::new("op_impl", operator_impl_parse),
                                    (OptField::new("table_select", table_select_parse), OptEnd),
                                ),
                            ),
                        ),
                    ),
                ),
            )
                .gen('=');
            let (_, res) = parser.comp(TokenIter::from(opts, backend_name.span()));
            res.to_result().map_err(TokenDiagnostic::into_list).map(
                |(
                    debug,
                    (
                        interface,
                        (
                            public,
                            (ds_name, (inline_queries, (operator_impl, (table_selector, ())))),
                        ),
                    ),
                )| Serialized {
                    debug,
                    interface,
                    public: public.unwrap_or(false),
                    ds_name,
                    aggressive_inlining: inline_queries.unwrap_or(false),
                    operator_impl: operator_impl.unwrap_or(DEFAULT_OP_IMPL),
                    table_selector: table_selector.unwrap_or(DEFAULT_TABLE_SELECTOR),
                },
            )
        } else {
            Ok(Self {
                debug: None,
                interface: None,
                public: false,
                ds_name: None,
                aggressive_inlining: false,
                operator_impl: DEFAULT_OP_IMPL,
                table_selector: DEFAULT_TABLE_SELECTOR,
            })
        }
    }

    fn generate_code(
        self,
        impl_name: syn::Ident,
        plan: &crate::plan::Plan,
    ) -> Result<proc_macro2::TokenStream, std::collections::LinkedList<proc_macro_error2::Diagnostic>>
    {
        let mut namer = namer::SerializedNamer::new();
        if let Some(name) = self.ds_name {
            namer.struct_datastore = name;
        }

        let tables::TableWindow {
            table_defs,
            datastore,
            datastore_impl,
            database,
            table_generated_info,
        } = tables::generate_tables(
            plan,
            &self.interface,
            &namer,
            &self.table_selector,
            self.aggressive_inlining,
        );

        let record_defs =
            types::generate_record_definitions(plan, &table_generated_info.get_types, &namer);

        let operator_impl = self.operator_impl.get_paths();

        let QueriesInfo {
            query_mod,
            query_impls,
            required_stats,
        } = queries::generate_queries(
            plan,
            &table_generated_info,
            &self.interface,
            &namer,
            &operator_impl,
            self.aggressive_inlining,
        );

        let namer::SerializedNamer { mod_tables, .. } = &namer;

        let public_tk = if self.public { quote!(pub) } else { quote!() };
        
        let stats_struct = required_stats.generate_stats_struct(&namer, &operator_impl);
        
        let minister_trait = operator_impl.trait_path;

        let tks = quote! {
            #public_tk mod #impl_name {
                // lints (generated code not idiomatic, and can propagate confusing/incorrect lints to user code)
                #![allow(non_shorthand_field_patterns)] // current name field printing is `fielname: fieldname`
                #![allow(unused_variables)]
                #![allow(dead_code)]

                use #minister_trait; //TODO: remove and use better operator selection
                pub mod #mod_tables {
                    #(#table_defs)*
                }
                #query_mod
                #(#record_defs)*
                #datastore
                #datastore_impl
                #database
                #query_impls
                #stats_struct
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
