use std::collections::LinkedList;

use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use syn::{
    parse2, punctuated::Punctuated, spanned::Spanned, Field, Fields, FieldsUnnamed, ItemEnum, Path,
    PathArguments, PathSegment, TypePath, Visibility,
};

use combi::{
    core::{mapsuc, nothing, seq},
    macros::seqs,
    tokens::{
        basic::{
            collectuntil, getident, isempty, matchident, matchpunct, recovgroup, syn, terminal,
        },
        TokenDiagnostic, TokenIter,
    },
    Combi, CombiResult,
};

use crate::passing::{extract_syn, CallStore, ItemInfo};

pub fn interface(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let macro_name = parse_attrs(attr)?;
    let enum_def = extract_syn(item, parse2::<ItemEnum>)?;
    let trans_enum = transform_enum(enum_def)?;

    let enum_tks = trans_enum.clone();
    let pass_tks = ItemInfo(trans_enum).store_grouped();

    // NOTE: similar to the merge pattern discussed in the documentation, except we do
    //        not need a second macro store (reduces the expansion depth required)
    Ok(quote! {
        #enum_tks
        macro_rules! #macro_name {
            ($p:path => $($t:tt)*) => {
                $p!( $($t)*  #pass_tks );
            }
        }
    })
}

fn parse_attrs(attr: TokenStream) -> Result<Ident, LinkedList<Diagnostic>> {
    let (_, res) = getident().comp(TokenIter::from(attr, Span::call_site()));
    res.to_result().map_err(TokenDiagnostic::into_list)
}

fn transform_enum(mut enum_def: ItemEnum) -> Result<ItemEnum, LinkedList<Diagnostic>> {
    let mut errors = LinkedList::new();

    for variant in &mut enum_def.variants {
        if let Fields::Unit = variant.fields {
            let mut punctlist = Punctuated::new();
            let mut segs = Punctuated::new();
            segs.push(PathSegment {
                ident: variant.ident.clone(),
                arguments: PathArguments::None,
            });

            punctlist.push(Field {
                attrs: Vec::new(),
                vis: Visibility::Inherited,
                mutability: syn::FieldMutability::None,
                ident: None,
                colon_token: None,
                ty: syn::Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: segs,
                    },
                }),
            });
            variant.fields = syn::Fields::Unnamed(FieldsUnnamed {
                paren_token: syn::token::Paren {
                    span: Group::new(Delimiter::Parenthesis, TokenStream::new()).delim_span(),
                },
                unnamed: punctlist,
            })
        } else if let Fields::Unnamed(fs) = &variant.fields {
            if !fs.unnamed.len() == 1 {
                errors.push_back(
                    Diagnostic::spanned(
                        fs.unnamed.span(),
                        Level::Error,
                        "Provided variants should be a single identifier of the type.".to_owned(),
                    )
                    .help(format!(
                        "Try `enum {} {{ .. {}(<type>), .. }}`",
                        enum_def.ident, variant.ident
                    )),
                );
            }
        } else {
            errors.push_back(
                Diagnostic::spanned(
                    variant.fields.span(),
                    Level::Error,
                    "Provided variants should be a single identifier of the type.".to_owned(),
                )
                .help(format!(
                    "Try `enum {} {{ .. {}, .. }}`",
                    enum_def.ident, variant.ident
                )),
            );
        }
    }

    if errors.is_empty() {
        Ok(enum_def)
    } else {
        Err(errors)
    }
}
