use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};
use quote::{quote, ToTokens};
use syn::{
    parse2, punctuated::Punctuated, spanned::Spanned, Field, FieldsUnnamed, FnArg, Generics, ItemEnum, ItemImpl, ItemTrait, Path, PathArguments, PathSegment, TypePath, Visibility
};

pub fn register(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Vec<Diagnostic>> {
    parse_attrs(attr)?;
    let EnumInfo {
        trait_args,
        macro_name,
        trans_item,
    } = parse_enum(item)?;
    let gen = generate_register_macro(macro_name, trait_args);
    Ok(quote! {
        #trans_item
        #gen
    })
}

fn parse_attrs(attr: TokenStream) -> Result<(), Vec<Diagnostic>> {
    if !attr.is_empty() {
        Err(vec![Diagnostic::spanned(
            attr.span(),
            Level::Error,
            "No extra arguments should be provided".to_owned(),
        )])
    } else {
        Ok(())
    }
}

struct TraitArgs {
    fields: Vec<Ident>,
    enum_name: Ident,
}

impl TraitArgs {
    fn to_tokens(self) -> TokenStream {
        let Self { fields, enum_name } = self;
        quote! {
            #enum_name {#(#fields)*}
        }
    }

    fn from_tokens(tks: TokenStream) -> Self {
        let mut tokens = tks.into_iter();
        let enum_name_raw = tokens.next().unwrap();
        let fields_raw = tokens.next().unwrap();
        assert!(tokens.next().is_none());
        
        Self { 
            fields: {
                extract_group(fields_raw, &Delimiter::Brace).into_iter().map(|t| match t {
                    TokenTree::Ident(i) => i,
                    _ => unreachable!("Expected an identifier"),
                }).collect()
            }, 
            enum_name: {
                match enum_name_raw {
                    TokenTree::Ident(i) => i.clone(),
                    _ => unreachable!("Expected an identifier"),
                }
            } 
        }
    }
}

struct EnumInfo {
    trait_args: TraitArgs,
    macro_name: Ident,
    trans_item: ItemEnum,
}

fn has_generics(gen: &Generics) -> bool {
    !gen.params.is_empty() || gen.where_clause.is_some()
}

fn extract_group(tt: TokenTree, delim: &Delimiter) -> TokenStream {
    match tt {
        TokenTree::Group(g) if g.delimiter() == *delim => g.stream(),
        _ => unreachable!("Expected a group with brace"),
    }
}

fn extract_syn<T>(tks: TokenStream, f: impl Fn(TokenStream) -> syn::Result<T>) -> Result<T, Vec<Diagnostic>> {
    match f(tks) {
        Ok(o) => Ok(o),
        Err(errs) => 
            Err(errs
                .into_iter()
                .map(|err| Diagnostic::spanned(err.span(), Level::Error, err.to_string()))
                .collect()
            )
        ,
    }
}

fn parse_enum(item: TokenStream) -> Result<EnumInfo, Vec<Diagnostic>> {
    let mut en = extract_syn(item, parse2::<ItemEnum>)?;

    let mut fields = Vec::new();
    let mut errors = Vec::new();

    for variant in &mut en.variants {
        if variant.fields.is_empty() {
            fields.push(variant.ident.clone());
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
                    span: Group::new(Delimiter::Parenthesis, TokenStream::new())
                        .delim_span(),
                },
                unnamed: punctlist,
            })
        } else {
            errors.push(
                Diagnostic::spanned(
                    variant.fields.span(),
                    Level::Error,
                    "Provided variants should be a single identifier of the type."
                        .to_owned(),
                )
                .help(format!(
                    "Try `enum {} {{ .. {}, .. }}`",
                    en.ident, variant.ident
                )),
            );
        }
    }

    if has_generics(&en.generics) {
        errors.push(Diagnostic::spanned(en.generics.span(), Level::Error, "Generics are not supported".to_owned()));
    }

    if errors.is_empty() {
        Ok(EnumInfo {
            trait_args: TraitArgs{
            fields,
            enum_name: en.ident.clone()},
            macro_name: Ident::new(&format!("{}_enumitem", en.ident), en.ident.span()),
            trans_item: en,
        })
    } else {
        Err(errors)
    }
}

fn generate_register_macro(
    macro_name: Ident, targs : TraitArgs
) -> TokenStream {
    let targs_tks = targs.to_tokens();
    quote! {
        macro_rules! #macro_name {
            ($($t:tt)*) => {
                enumtrait::implement!({$($t)*} {#targs_tks});
            }
        }
    }
}

fn generate_impl(TraitArgs { fields, enum_name }: TraitArgs, trait_item: &ItemTrait) -> Result<ItemImpl, Vec<Diagnostic>> {
    let mut impls = Vec::new();
    let mut errors = Vec::new();
    
    fn unsupported(errors: &mut Vec<Diagnostic>, span: Span, kind: &str) {
        errors.push(Diagnostic::spanned(span, Level::Error, format!("{kind} are not supported by enumtrait")))
    }

    for item in &trait_item.items {
        match item {
            syn::TraitItem::Const(c) => unsupported(&mut errors, c.span(), "Constants"),
            syn::TraitItem::Type(t) => unsupported(&mut errors, t.span(), "Types"),
            syn::TraitItem::Macro(m) => unsupported(&mut errors, m.span(), "Macros"),
            syn::TraitItem::Verbatim(ts) => unsupported(&mut errors, ts.span(), "Arbitrary tokens"),
            syn::TraitItem::Fn(f) => {
                if !matches!(f.sig.inputs.first(), Some(&FnArg::Receiver(_))) {
                    errors.push(Diagnostic::spanned(f.sig.inputs.span(), Level::Error, "All trait functions need to start with a recieved (e.g. &self, &mut self, self)".to_owned()));
                } else {
                    impls.push(&f.sig)
                }
            },
            other => unsupported(&mut errors, Span::call_site(), "Unsupported trait item"),
        }
    }

    quote! {

    }

    todo!()
}

// INV: the tokenstream is of form { <possibly trait> } { <TraitArgs> }
pub fn implement(tks: TokenStream) -> Result<TokenStream, Vec<Diagnostic>> {
    let mut items = tks.into_iter();
    let (trait_item, targs) = (extract_group(items.next().unwrap(), &Delimiter::Brace), extract_group(items.next().unwrap(), &Delimiter::Brace));
    assert!(items.next().is_none()); // only two items from INV
    
    let TraitArgs { fields, enum_name } = TraitArgs::from_tokens(targs);
    let trait_def = extract_syn(trait_item, parse2::<ItemTrait>)?;


    
    Ok(TokenStream::new())
}


