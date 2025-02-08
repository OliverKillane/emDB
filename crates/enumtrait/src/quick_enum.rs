use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream};
use proc_macro_error2::{Diagnostic, Level};
use quote::quote;
use std::collections::LinkedList;
use syn::{
    parse2, punctuated::Punctuated, spanned::Spanned, Field, Fields, FieldsUnnamed, ItemEnum, Path,
    PathArguments, PathSegment, TypePath, Visibility,
};

use crate::macro_comm::extract_syn;

pub fn interface(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, LinkedList<Diagnostic>> {
    parse_attrs(attr)?;
    let enum_def = extract_syn(
        item,
        &Ident::new("quick_enum", Span::call_site()),
        parse2::<ItemEnum>,
    )?;
    let pass_tks = transform_enum(enum_def)?;
    Ok(quote! { #pass_tks })
}

fn parse_attrs(attr: TokenStream) -> Result<(), LinkedList<Diagnostic>> {
    if attr.is_empty() {
        Ok(())
    } else {
        Err(LinkedList::from([Diagnostic::spanned(
            attr.span(),
            Level::Error,
            "This macro takes no arguments".to_owned(),
        )]))
    }
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
            });
        } else if let Fields::Unnamed(fs) = &variant.fields {
            if fs.unnamed.len() != 1 {
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
