use combi::{
    tokens::{basic::getident, TokenDiagnostic, TokenIter},
    Combi,
};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::Diagnostic;
use quote::quote;
use std::collections::LinkedList;

pub fn interface(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let macro_name = parse_attrs(attr)?;

    Ok(quote! {
        #item
        macro_rules! #macro_name {
            (expr_ctx $p:ident => $($t:tt)*) => { // if expression context, no ; is required
                $p!( $($t)* { #macro_name { #item } } )
            };
            (item_ctx $p:ident => $($t:tt)*) => { // if being used in an item context, need to end with ; or use macro!{ }
                $p!( $($t)* { #macro_name { #item } } );
            };
        }
    })
}

fn parse_attrs(attr: TokenStream) -> Result<Ident, LinkedList<Diagnostic>> {
    let (_, res) = getident().comp(TokenIter::from(attr, Span::call_site()));
    res.to_result().map_err(TokenDiagnostic::into_list)
}
