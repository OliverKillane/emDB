//! Forwards rust expressions from the plan in order to check their code, when no backend impl is needed.
//! - Can be used for debugging.
//! - less costly, can run with no optimisers.
//! - useful for tests with no artifacts

use std::{collections::LinkedList, fs::File, io::Write, path::Path};

use crate::{
    analysis::interface::{names::SimpleNamer, types::{SimpleTypeImplementor, TypeImplementor}}, utils::misc::singlelist
};

use proc_macro2::TokenStream;
use crate::{analysis::interface::{contexts::{trans_context, ClosureArgs}, names::{ItemNamer}}, plan};

use super::EMDBBackend;
use combi::{core::{mapsuc, seq, setrepr}, seqs, tokens::{basic::{collectuntil, isempty, matchident, matchpunct, syn}, error::expectederr, TokenDiagnostic, TokenIter}, Combi};
use proc_macro_error::{Diagnostic, Level};
use syn::{parse2, File as SynFile, LitStr};
use quote::quote;
use prettyplease::unparse;

pub struct SemCheck {
    debug: Option<LitStr>
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
        let ty_impl = SimpleTypeImplementor::<SimpleNamer>::with_public_types(plan);
        let types_preamble = ty_impl.translate_all_types(plan);
        let queries = translate_all_queries(plan);


        let tks = quote! {
            mod #impl_name { 
                #![allow(unused_variables)]
                #![allow(dead_code)]

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

fn debug_output(debug_path: &LitStr, tks: TokenStream) -> Result<(), LinkedList<Diagnostic>> {
    match parse2::<SynFile>(tks) {
        Ok(m) => {
            match File::create(Path::new(&debug_path.value())) {
                Ok(mut f) => {
                    match f.write_all(unparse(&m).as_bytes()) {
                        Ok(()) => Ok(()),
                        Err(e) => Err(singlelist(Diagnostic::spanned(debug_path.span(), Level::Error, format!("Could not write to file: {e}")))),
                    }
                },
                Err(e) => Err(singlelist(Diagnostic::spanned(debug_path.span(), Level::Error, format!("Could not create file: {e}"))))
            }
        },
        Err(e) => Err(singlelist(Diagnostic::spanned(debug_path.span(), Level::Error, format!("Could not parse code as file: {e}")))),
    }
}

pub fn translate_all_queries(lp: &plan::Plan) -> TokenStream {
    lp.queries.iter().map(|(key, query)| translate_query(lp, key, query)).collect()
}

fn translate_query(lp: &plan::Plan, qk: plan::Key<plan::Query>, query: &plan::Query) -> TokenStream {
    let ClosureArgs { params, value } = trans_context::<SimpleNamer>(lp, query.ctx);

    let query_params = params.iter().map(|(id, ty)| {
        quote! { #id: #ty }
    });
    let query_name = &query.name;
    let query_closure_gen = value.expression;
    let query_closure_type = value.datatype;

    let return_type = if let Some(ret_op) = lp.get_context(query.ctx).returnflow {
        let ret = lp.get_operator(ret_op).get_return();
        let ret_type = SimpleNamer::record_type(lp.get_dataflow(ret.input).get_conn().with.fields);
        quote! { -> #ret_type }
    } else {
        quote!()
    };

    quote!{
        /// this is a function
        pub fn #query_name(#(#query_params ,)*) #return_type {
            let closures = #query_closure_gen ;
            
            todo!()
        }
    }
}