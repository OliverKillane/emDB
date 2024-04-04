use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use syn::{
    parse2, punctuated::Punctuated, spanned::Spanned, Field, Fields, FieldsUnnamed, ItemEnum, Path,
    PathArguments, PathSegment, TypePath, Visibility,
};

use crate::passing::{extract_syn, ItemInfo, CallStore};

// Generate a macro and modified implementation
pub fn interface(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Vec<Diagnostic>> {
    let macro_name = parse_attrs(attr)?;
    let enum_def = extract_syn(item, parse2::<ItemEnum>)?;
    let enum_info = transform_enum(enum_def)?;

    let enum_src_tks = enum_info.0.clone();
    let pass_tks = enum_info.store_grouped(); 

    Ok(quote! {
        #enum_src_tks
        macro_rules! #macro_name {
            ($($t:tt)*) => {
                enumtrait::get_trait_apply!( $($t)*  #pass_tks );
            }
        }
    })
}

fn parse_attrs(attr: TokenStream) -> Result<Ident, Vec<Diagnostic>> {
    let mut attr_iter = attr.into_iter();
    if let Some(tk) = attr_iter.next() {
        let macro_name = match tk {
            TokenTree::Ident(i) => Ok(i),
            _ => Err(vec![Diagnostic::spanned(
                tk.span(),
                Level::Error,
                "Expected an identifier".to_owned(),
            )
            .help("#[enumtrait::register(name_of_macro)]".to_owned())]),
        }?;

        if let Some(tk) = attr_iter.next() {
            Err(vec![Diagnostic::spanned(
                tk.span(),
                Level::Error,
                "No extra arguments should be provided".to_owned(),
            )])
        } else {
            Ok(macro_name)
        }
    } else {
        Err(vec![Diagnostic::spanned(
            Span::call_site(),
            Level::Error,
            "No arguments provided".to_owned(),
        )])
    }
}

fn transform_enum(mut enum_def: ItemEnum) -> Result<ItemInfo<ItemEnum>, Vec<Diagnostic>> {
    let mut errors = Vec::new();

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
                errors.push(
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
            errors.push(
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
        Ok(ItemInfo(enum_def))
    } else {
        Err(errors)
    }
}
