use std::collections::LinkedList;

use crate::macro_comm::{extract_syn, CallStore, ItemInfo, Triple};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use quote::{quote, ToTokens};
use syn::{
    parse2, punctuated::Punctuated, spanned::Spanned, token::{Brace, Comma, Dot, FatArrow, Match, Paren, SelfValue}, Arm, Block, Expr, ExprMatch, ExprMethodCall, ExprPath, FnArg, ImplItem, ImplItemFn, ItemEnum, ItemImpl, ItemTrait, Pat, PatIdent, PatTupleStruct, Path, PathSegment, Signature, Stmt, TraitItem, TraitItemFn
};

use combi::{
    core::{mapsuc, seq},
    macros::seqs,
    tokens::{
        basic::{collectuntil, isempty, matchident, peekident},
        TokenDiagnostic, TokenIter,
    },
    Combi,
};

pub fn interface(
    attrs: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let (trait_macro_store, enum_macro_store) = parse_attrs(attrs)?;
    let invoke_ident = Ident::new("impl_trait", Span::call_site());
    let trait_item = ItemInfo {
        data: extract_syn(item.clone(), &invoke_ident, parse2::<ItemImpl>)?,
        label: invoke_ident,
    }
    .store_grouped();

    let tks = quote! {
        #enum_macro_store!( item_ctx #trait_macro_store => item_ctx enumtrait::impl_trait_apply => #trait_item ) ;
    };
    Ok(tks)
}

fn parse_attrs(attrs: TokenStream) -> Result<(TokenStream, TokenStream), LinkedList<Diagnostic>> {
    let parser = mapsuc(
        seqs!(collectuntil(peekident("for")), matchident("for"), collectuntil(isempty())),
        |(trait_macro_store, (_, enum_macro_store))| (trait_macro_store, enum_macro_store),
    );

    let (_, res) = parser.comp(TokenIter::from(attrs, Span::call_site()));
    res.to_result().map_err(TokenDiagnostic::into_list)
}

pub fn apply(input: TokenStream) -> Result<TokenStream, LinkedList<Diagnostic>> {
    let Triple(
        ItemInfo {
            data: impl_item,
            label: _,
        },
        ItemInfo {
            data: enum_item,
            label: _,
        },
        ItemInfo {
            data: trait_item,
            label: _,
        },
    ) = Triple::read(input)?;

    Ok(add_fn_impls(impl_item, trait_item, enum_item)?.into_token_stream())
}

fn add_fn_impls(mut impl_item: ItemImpl, trait_item: ItemTrait, enum_item: ItemEnum) -> Result<ItemImpl, LinkedList<Diagnostic>> {
    let qualified_name = get_path(&impl_item)?;
    for item in trait_item.items {
        if let TraitItem::Fn(ref f_item) = item {
            if let Some(gen_impl) = generate_fn_impl(f_item, &qualified_name, &enum_item) {
                impl_item.items.push(ImplItem::Fn(gen_impl));
            }
        }
    }

    Ok(impl_item)
}

fn get_path(impl_item: &ItemImpl) -> Result<Path, LinkedList<Diagnostic>> {
    match impl_item.self_ty.as_ref() {
        syn::Type::Path(ref p) => {
            // NOTE: as we are generating a path to use in a pattern match, we cannot allow paths 
            //       with arguments, unless through turbofish. As these can be inferred trivially 
            //       from `match self {..}` we can just strip them out.
            
            let mut path = Path {
                leading_colon: p.path.leading_colon,
                segments: Punctuated::new(),
            };

            for seg in p.path.segments.iter() {
                path.segments.push(PathSegment {
                    ident: seg.ident.clone(),
                    arguments: syn::PathArguments::None,
                })
            }
            Ok(path)
        },
        r => Err(LinkedList::from([Diagnostic::spanned(
            r.span(),
            Level::Error,
            "Expected a qualified type".to_owned(),
        )])),
    }
}

fn extract_params(sig: Signature) -> Option<(SelfValue, Vec<Ident>)> {
    let mut x = sig.inputs.into_iter();
    if let Some(FnArg::Receiver(r)) = x.next() {
        Some((r.self_token,
        x.map(|arg| {
            if let FnArg::Typed(pt) = arg{
                if let Pat::Ident(arg_pt) = *pt.pat {
                    arg_pt.ident
                } else {
                    unreachable!("Cannot have patterns in a trait argument but found {pt:?}")
                }
            } else {
                unreachable!("Cannot have a receiver past the first argument in a trait function but found {arg:?}")
            }
        }).collect()
        ))
    } else {
        None
    }
}

fn generate_fn_impl(trait_fn: &TraitItemFn, enum_qual: &Path, enum_item: &ItemEnum) -> Option<ImplItemFn> {
    let (self_token, args) = extract_params(trait_fn.sig.clone())?;

    let pat_expr = Ident::new("it", Span::call_site());

    let mut args_exprs = Punctuated::new();
    args_exprs.push(Expr::Path(ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: Path {
            leading_colon: None,
            segments: args
                .iter()
                .map(|arg| PathSegment {
                    ident: arg.clone(),
                    arguments: syn::PathArguments::None,
                })
                .collect(),
        },
    }));

    let expr_match = ExprMatch {
        attrs: Vec::new(),
        match_token: Match {
            span: Span::call_site(),
        },
        expr: Box::new(Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: self_token.into(),
        })),
        brace_token: Brace::default(),
        arms: enum_item
            .variants
            .iter()
            .map(|var| {
                // pat path is Something::Name ( it )
                let mut pat_path = enum_qual.clone();
                pat_path.segments.push(PathSegment {
                    ident: var.ident.clone(),
                    arguments: syn::PathArguments::None,
                });

                let mut pat_elems = Punctuated::new();
                pat_elems.push_value(Pat::Ident(PatIdent {
                    attrs: Vec::new(),
                    by_ref: None,
                    mutability: None,
                    ident: pat_expr.clone(),
                    subpat: None,
                }));

                let mut pat_reciever = Punctuated::new();
                pat_reciever.push_value(PathSegment {
                    ident: pat_expr.clone(),
                    arguments: syn::PathArguments::None,
                });

                Arm {
                    attrs: Vec::new(),
                    pat: Pat::TupleStruct(PatTupleStruct {
                        attrs: Vec::new(),
                        qself: None,
                        path: pat_path,
                        paren_token: Paren::default(),
                        elems: pat_elems,
                    }),
                    guard: None,
                    fat_arrow_token: FatArrow::default(),
                    body: Box::new(Expr::MethodCall(ExprMethodCall {
                        attrs: Vec::new(),
                        receiver: Box::new(Expr::Path(ExprPath {
                            attrs: Vec::new(),
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: pat_reciever,
                            },
                        })),
                        dot_token: Dot::default(),
                        method: trait_fn.sig.ident.clone(),
                        turbofish: None,
                        paren_token: Paren::default(),
                        args: args_exprs.clone(),
                    })),
                    comma: Some(Comma::default()),
                }
            })
            .collect(),
    };

    Some(ImplItemFn {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: trait_fn.sig.clone(),
        block: Block {
            brace_token: expr_match.brace_token,
            stmts: vec![Stmt::Expr(Expr::Match(expr_match), None)],
        },
    })
}
