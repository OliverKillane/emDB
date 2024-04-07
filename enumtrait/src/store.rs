use combi::{
    choices, core::{choice, mapsuc, seq}, macros::seqs, tokens::{basic::{getident, matchident, peekident}, TokenDiagnostic, TokenIter}, Combi
};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::Diagnostic;
use quote::quote;
use std::collections::LinkedList;

pub fn interface(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let (macro_name, vis) = parse_attrs(attr)?;

    let macro_tks = quote! {
        macro_rules! #macro_name {
            (expr_ctx $p:ident => $($t:tt)*) => { // if expression context, no ; is required
                $p!( $($t)* { #macro_name { #item } } )
            };
            (item_ctx $p:path => $($t:tt)*) => { // if being used in an item context, need to end with ; or use macro!{ }
                $p!( $($t)* { #macro_name { #item } } );
            };
        }
    };

    let (export_tks, pub_tks) = match vis {
        Attr::Pub(p) => {
            (
                quote!{},
                quote!{#p(crate) use #macro_name;}
            )
        },
        Attr::Export(e) => {
            let export = Ident::new("macro_export", e.span());
            (
                quote!{#[#export]},
                quote!{pub use #macro_name;}
            )
        },
        Attr::None => (quote!(), quote!())
    };


    Ok(
        quote! {
            #item
            #export_tks
            #macro_tks
            #pub_tks
        }
    )
}

enum Attr {
    Pub(Ident),
    Export(Ident),
    None,
}

fn parse_attrs(attr: TokenStream) -> Result<(Ident, Attr), LinkedList<Diagnostic>> {
    let parser = 
        choices! {
            peekident("pub") => mapsuc(seqs!(matchident("pub"), getident()), |(p, i)| (i, Attr::Pub(p))),
            peekident("export") => mapsuc(seqs!(matchident("export"), getident()), |(e, i)| (i, Attr::Export(e))),
            otherwise => mapsuc(getident(), |i| (i, Attr::None))
        };
    let (_, res) = parser.comp(TokenIter::from(attr, Span::call_site()));
    res.to_result().map_err(TokenDiagnostic::into_list)
}
