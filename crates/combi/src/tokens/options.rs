//! A combi parser for parsing a structure out of order.

use std::{
    collections::{HashMap, LinkedList},
    marker::PhantomData,
};

use crate::{
    core::{choice, mapall, mapsuc, seq, seqdiff, DiffRes},
    tokens::{
        basic::{collectuntil, getident, gettoken, matchpunct, peekident, peekpunct, terminal},
        derived::listseptrailing,
        error::error,
        TokenDiagnostic, TokenIter, TokenParser,
    },
    Combi, CombiResult,
};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use syn::Ident;

pub trait OptParse: Sized {
    type Curr;
    type Rest;
    type All;

    fn construct(
        self,
        sep_tk: char,
        prev: impl TokenParser<(Ident, TokenStream)>,
    ) -> impl TokenParser<(Self::All, HashMap<Ident, TokenStream>)>;

    fn error_key(&self, options: &mut Vec<&'static str>);

    fn gen(self, sep_tk: char) -> impl TokenParser<Self::All> {
        let mut options = Vec::new();
        self.error_key(&mut options);
        let options_available = options.join(", ");
        let options_available2 = options_available.clone();
        mapall(
            self.construct(
                sep_tk,
                error(gettoken, move |t| {
                    Diagnostic::spanned(
                        t.span(),
                        Level::Error,
                        format!("Expected {options_available}"),
                    )
                }),
            ),
            move |(value, others)| {
                let errors = others
                    .into_keys()
                    .map(|k| {
                        Diagnostic::spanned(
                            k.span(),
                            Level::Error,
                            format!("{k} is not available, must be one of: {options_available2}"),
                        )
                    })
                    .collect::<LinkedList<_>>();
                if errors.is_empty() {
                    CombiResult::Suc(value)
                } else {
                    CombiResult::Err(
                        TokenDiagnostic::from_list(errors).expect(
                            "Non-empty, so at least one element, so we must have a diagnostic",
                        ),
                    )
                }
            },
        )
    }
}

pub struct OptEnd;

impl OptParse for OptEnd {
    type Curr = ();
    type Rest = ();
    type All = ();

    fn construct(
        self,
        _sep_tk: char,
        prev: impl TokenParser<(Ident, TokenStream)>,
    ) -> impl TokenParser<(Self::All, HashMap<Ident, TokenStream>)> {
        mapall(listseptrailing(',', prev), |values| {
            let mut uniques: HashMap<Ident, TokenStream> = HashMap::new();
            let mut errors = LinkedList::new();
            for (key, value) in values {
                if let Some((k2, _)) = uniques.get_key_value(&key) {
                    errors.push_back(
                        Diagnostic::spanned(
                            key.span(),
                            Level::Error,
                            format!("Duplicate option `{key}`"),
                        )
                        .span_error(k2.span(), String::from("originally defined here")),
                    )
                } else {
                    uniques.insert(key, value);
                }
            }
            if errors.is_empty() {
                CombiResult::Suc(((), uniques))
            } else {
                CombiResult::Err(
                    TokenDiagnostic::from_list(errors)
                        .expect("Non-empty, so at least one element, so we must have a diagnostic"),
                )
            }
        })
    }

    fn error_key(&self, _options: &mut Vec<&'static str>) {}
}

pub struct OptField<O, P: TokenParser<O>, F: Fn() -> P> {
    name: &'static str,
    parser: F,
    phantom: PhantomData<O>,
}

impl<O, P: TokenParser<O>, F: Fn() -> P> OptField<O, P, F> {
    pub fn new(name: &'static str, parser: F) -> Self {
        Self {
            name,
            parser,
            phantom: PhantomData,
        }
    }
}

impl<O, P: TokenParser<O>, R: OptParse, F: Fn() -> P> OptParse for (OptField<O, P, F>, R) {
    type Curr = O;
    type Rest = R::All;
    type All = (Option<O>, R::All);

    fn construct(
        self,
        sep_tk: char,
        prev: impl TokenParser<(Ident, TokenStream)>,
    ) -> impl TokenParser<(Self::All, HashMap<Ident, TokenStream>)> {
        let (
            OptField {
                name,
                parser,
                phantom: _,
            },
            rest,
        ) = self;

        mapall(
            rest.construct(
                sep_tk,
                choice(
                    peekident(name),
                    seq(
                        mapsuc(seq(getident(), matchpunct(sep_tk)), |(k, _)| k),
                        collectuntil(peekpunct(',')),
                    ),
                    prev,
                ),
            ),
            move |(rest, mut uniques)| {
                if let Some((key, _)) = uniques.get_key_value(&Ident::new(name, Span::call_site()))
                {
                    let key = key.clone();
                    let val = uniques
                        .remove(&key)
                        .expect("Key was use for access already taken from the map");

                    match (seqdiff(parser(), terminal)).comp(TokenIter::from(val, key.span())) {
                        (DiffRes::First(_), CombiResult::Suc(_)) => {
                            unreachable!("Would pass to second")
                        } // TODO: find nicer way around this from combi
                        (DiffRes::Second(()), CombiResult::Suc((val, ()))) => {
                            CombiResult::Suc(((Some(val), rest), uniques))
                        }
                        (DiffRes::First(_), CombiResult::Con(c)) => CombiResult::Con(c),
                        (DiffRes::First(_), CombiResult::Err(e)) => CombiResult::Err(e),
                        (DiffRes::Second(()), CombiResult::Con(c)) => CombiResult::Con(c),
                        (DiffRes::Second(()), CombiResult::Err(e)) => CombiResult::Err(e),
                    }
                } else {
                    CombiResult::Suc(((None, rest), uniques))
                }
            },
        )
    }

    fn error_key(&self, options: &mut Vec<&'static str>) {
        options.push(self.0.name);
        self.1.error_key(options);
    }
}

pub struct MustField<O, P: TokenParser<O>, F: Fn() -> P> {
    name: &'static str,
    parser: F,
    phantom: PhantomData<O>,
}

impl<O, P: TokenParser<O>, F: Fn() -> P> MustField<O, P, F> {
    pub fn new(name: &'static str, parser: F) -> Self {
        Self {
            name,
            parser,
            phantom: PhantomData,
        }
    }
}

impl<O, P: TokenParser<O>, R: OptParse, F: Fn() -> P> OptParse for (MustField<O, P, F>, R) {
    type Curr = O;
    type Rest = R::All;
    type All = (O, R::All);

    fn construct(
        self,
        sep_tk: char,
        prev: impl TokenParser<(Ident, TokenStream)>,
    ) -> impl TokenParser<(Self::All, HashMap<Ident, TokenStream>)> {
        let (
            MustField {
                name,
                parser,
                phantom: _,
            },
            rest,
        ) = self;

        mapall(
            rest.construct(
                sep_tk,
                choice(
                    peekident(name),
                    seq(
                        mapsuc(seq(getident(), matchpunct(sep_tk)), |(k, _)| k),
                        collectuntil(peekpunct(',')),
                    ),
                    prev,
                ),
            ),
            move |(rest, mut uniques)| {
                if let Some((key, _)) = uniques.get_key_value(&Ident::new(name, Span::call_site()))
                {
                    let key = key.clone();
                    let val = uniques
                        .remove(&key)
                        .expect("Key was use for access already taken from the map");

                    match (seqdiff(parser(), terminal)).comp(TokenIter::from(val, key.span())) {
                        (DiffRes::First(_), CombiResult::Suc(_)) => {
                            unreachable!("Would pass to second")
                        } // TODO: find nicer way around this from combi
                        (DiffRes::Second(()), CombiResult::Suc((val, ()))) => {
                            CombiResult::Suc(((val, rest), uniques))
                        }
                        (DiffRes::First(_), CombiResult::Con(c)) => CombiResult::Con(c),
                        (DiffRes::First(_), CombiResult::Err(e)) => CombiResult::Err(e),
                        (DiffRes::Second(()), CombiResult::Con(c)) => CombiResult::Con(c),
                        (DiffRes::Second(()), CombiResult::Err(e)) => CombiResult::Err(e),
                    }
                } else {
                    CombiResult::Con(TokenDiagnostic::from(Diagnostic::spanned(
                        Span::call_site(),
                        Level::Error,
                        format!("Missing required field `{name}`"),
                    )))
                }
            },
        )
    }

    fn error_key(&self, options: &mut Vec<&'static str>) {
        options.push(self.0.name);
        self.1.error_key(options);
    }
}

pub struct DefaultField<O, P: TokenParser<O>, F: Fn() -> P, D: Fn() -> O> {
    name: &'static str,
    parser: F,
    default: D,
    phantom: PhantomData<O>,
}

impl<O, P: TokenParser<O>, F: Fn() -> P, D: Fn() -> O> DefaultField<O, P, F, D> {
    pub fn new(name: &'static str, parser: F, default: D) -> Self {
        Self {
            name,
            parser,
            default,
            phantom: PhantomData,
        }
    }
}

impl<O, P: TokenParser<O>, R: OptParse, F: Fn() -> P, D: Fn() -> O> OptParse
    for (DefaultField<O, P, F, D>, R)
{
    type Curr = O;
    type Rest = R::All;
    type All = (O, R::All);

    fn construct(
        self,
        sep_tk: char,
        prev: impl TokenParser<(Ident, TokenStream)>,
    ) -> impl TokenParser<(Self::All, HashMap<Ident, TokenStream>)> {
        let (
            DefaultField {
                name,
                parser,
                default,
                phantom: _,
            },
            rest,
        ) = self;

        mapall(
            rest.construct(
                sep_tk,
                choice(
                    peekident(name),
                    seq(
                        mapsuc(seq(getident(), matchpunct(sep_tk)), |(k, _)| k),
                        collectuntil(peekpunct(',')),
                    ),
                    prev,
                ),
            ),
            move |(rest, mut uniques)| {
                if let Some((key, _)) = uniques.get_key_value(&Ident::new(name, Span::call_site()))
                {
                    let key = key.clone();
                    let val = uniques
                        .remove(&key)
                        .expect("Key was use for access already taken from the map");

                    match (seqdiff(parser(), terminal)).comp(TokenIter::from(val, key.span())) {
                        (DiffRes::First(_), CombiResult::Suc(_)) => {
                            unreachable!("Would pass to second")
                        } // TODO: find nicer way around this from combi
                        (DiffRes::Second(()), CombiResult::Suc((val, ()))) => {
                            CombiResult::Suc(((val, rest), uniques))
                        }
                        (DiffRes::First(_), CombiResult::Con(c)) => CombiResult::Con(c),
                        (DiffRes::First(_), CombiResult::Err(e)) => CombiResult::Err(e),
                        (DiffRes::Second(()), CombiResult::Con(c)) => CombiResult::Con(c),
                        (DiffRes::Second(()), CombiResult::Err(e)) => CombiResult::Err(e),
                    }
                } else {
                    CombiResult::Suc(((default(), rest), uniques))
                }
            },
        )
    }

    fn error_key(&self, options: &mut Vec<&'static str>) {
        options.push(self.0.name);
        self.1.error_key(options);
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens::basic::matchident;

    use super::*;
    use quote::quote;

    fn get_result<T>(
        parser: &impl TokenParser<T>,
        input: TokenStream,
    ) -> Result<T, TokenDiagnostic> {
        parser
            .comp(TokenIter::from(input, Span::call_site()))
            .1
            .to_result()
    }

    #[test]
    fn basic_parse() {
        let config_opts = (
            OptField::new("foo", || mapsuc(getident(), |_| true)),
            (OptField::new("bar", getident), OptEnd),
        )
            .gen(':');

        let input1 = quote! {
            foo: foo,
            bar: bar,
        };

        let input2 = quote! {
            bar: bar,
            foo: foo,
        };

        let (_, (_, ())) = get_result(&config_opts, input1).unwrap();
        let (_, (_, ())) = get_result(&config_opts, input2).unwrap();
    }

    #[test]
    fn must_parse() {
        let config_opts = (
            OptField::new("foo", || mapsuc(getident(), |_| true)),
            (
                OptField::new("bar", getident),
                (MustField::new("baz", || matchident("bazingah")), OptEnd),
            ),
        )
            .gen(':');

        let input1 = quote! {
            foo: foo,
            bar: bar,
            baz: bazingah,
        };

        let input2 = quote! {
            bar: bar,
            baz: bazingah,
            foo: foo,
        };

        let error1 = quote! {
            bar: bar,
            foo: foo,
        };

        let (_, (_, (_, ()))) = get_result(&config_opts, input1).unwrap();
        let (_, (_, (_, ()))) = get_result(&config_opts, input2).unwrap();

        assert!(get_result(&config_opts, error1).is_err());
    }
}
