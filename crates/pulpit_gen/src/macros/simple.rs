//! # A simple interface for generating tables.

use std::collections::{HashMap, HashSet, LinkedList};

use combi::{
    core::{choice, mapsuc, nothing, recover, seq, seqdiff},
    logical::or,
    macros::{choices, seqs},
    tokens::{
        basic::{
            collectuntil, getident, gettoken, matchident, matchpunct, peekident, peekpunct,
            recovgroup, terminal,
        },
        derived::listseptrailing,
        error::{error, expectederr},
        recovery::until,
        TokenDiagnostic, TokenIter, TokenParser,
    },
    Combi,
};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use syn::Ident;

use crate::{
    groups::Field, operations::update::Update, predicates::Predicate, selector::SelectOperations,
    uniques::Unique,
};

struct ASTField {
    field_kind: Field,
    unique: Option<Ident>,
}

fn comma_after<T>(inp: impl TokenParser<T>) -> impl TokenParser<T> {
    mapsuc(
        seq(recover(inp, until(peekpunct(','))), matchpunct(',')),
        |(fields, _)| fields,
    )
}

fn fields_parser() -> impl TokenParser<Vec<ASTField>> {
    let unique_parse = mapsuc(
        seqs!(
            matchpunct('@'),
            matchident("unique"),
            recovgroup(proc_macro2::Delimiter::Parenthesis, getident())
        ),
        |(_, (_, id))| Some(id),
    );
    let inner = listseptrailing(
        ',',
        mapsuc(
            seqs!(
                getident(),
                matchpunct(':'),
                collectuntil(or(peekpunct(','), peekpunct('@'))),
                choice(peekpunct('@'), unique_parse, mapsuc(nothing(), |()| None))
            ),
            |(name, (_, (ty, unique)))| ASTField {
                field_kind: Field {
                    name,
                    ty: ty.into(),
                },
                unique,
            },
        ),
    );
    expectederr(named_parse(
        "fields",
        recovgroup(proc_macro2::Delimiter::Brace, inner),
    ))
}

fn parse_on_off(name: &'static str) -> impl TokenParser<bool> {
    mapsuc(
        seqs!(
            matchident(name),
            matchpunct(':'),
            choices!(
                peekident("on") => mapsuc(matchident("on"), |_| true),
                peekident("off") => mapsuc(matchident("off"), |_| false),
                otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, "Expected `on` or `off`".to_owned()))
            )
        ),
        |(_, (_, switch))| switch,
    )
}

fn parse_updates() -> impl TokenParser<Vec<Update>> {
    let update_parser = mapsuc(
        seqs!(
            getident(),
            matchpunct(':'),
            recovgroup(
                proc_macro2::Delimiter::Bracket,
                listseptrailing(',', getident())
            )
        ),
        |(alias, (_, fields))| Update { alias, fields },
    );
    named_parse(
        "updates",
        recovgroup(
            proc_macro2::Delimiter::Brace,
            listseptrailing(',', update_parser),
        ),
    )
}

fn parse_predicates() -> impl TokenParser<Vec<Predicate>> {
    let predicate_parser = mapsuc(
        seqs!(getident(), matchpunct(':'), collectuntil(peekpunct(','))),
        |(alias, (_, tokens))| Predicate {
            alias,
            tokens: tokens.into(),
        },
    );
    named_parse(
        "predicates",
        recovgroup(
            proc_macro2::Delimiter::Brace,
            listseptrailing(',', predicate_parser),
        ),
    )
}

fn named_parse<T>(name: &'static str, inner: impl TokenParser<T>) -> impl TokenParser<T> {
    mapsuc(seq(matchident(name), inner), |(_, data)| data)
}

fn analyse(
    fields: Vec<ASTField>,
    updates: Vec<Update>,
    predicates: Vec<Predicate>,
    transactions: bool,
    deletions: bool,
    name: Ident,
) -> Result<SelectOperations, LinkedList<Diagnostic>> {
    let mut seen_access_names: HashSet<Ident> = HashSet::new();
    let mut field_types = HashMap::new();
    let mut uniques = Vec::new();
    let mut errors = LinkedList::new();

    let mut add_duplicate = |curr_name: &Ident, prev_name: &Ident| {
        errors.push_back(
            Diagnostic::spanned(curr_name.span(), Level::Error, String::from("AAA"))
                .span_help(prev_name.span(), String::from("Originally here")),
        );
    };

    for pred in &predicates {
        if let Some(prev_name) = seen_access_names.get(&pred.alias) {
            add_duplicate(&pred.alias, prev_name)
        } else {
            seen_access_names.insert(pred.alias.clone());
        }
    }

    for ASTField { field_kind, unique } in fields {
        if let Some(alias) = unique {
            if let Some(name) = seen_access_names.get(&alias) {
                add_duplicate(&alias, name);
            } else {
                seen_access_names.insert(alias.clone());
            }
            uniques.push(Unique {
                alias,
                field: field_kind.name.clone(),
            })
        }

        if let Some(name) = seen_access_names.get(&field_kind.name) {
            add_duplicate(&field_kind.name, name);
        } else {
            seen_access_names.insert(field_kind.name.clone());
            field_types.insert(field_kind.name, field_kind.ty);
        }
    }

    Ok(SelectOperations {
        name,
        transactions,
        deletions,
        fields: field_types,
        uniques,
        predicates,
        updates,
        public: false,
    })
}

pub fn simple(input: TokenStream) -> Result<SelectOperations, LinkedList<Diagnostic>> {
    let parser = seqs!(
        comma_after(fields_parser()),
        comma_after(parse_updates()),
        comma_after(parse_predicates()),
        comma_after(parse_on_off("transactions")),
        comma_after(parse_on_off("deletions")),
        mapsuc(
            seqs!(matchident("name"), matchpunct(':'), getident()),
            |(_, (_, name))| name
        )
    );

    let (_, res) = mapsuc(seqdiff(parser, terminal), |(o, ())| o)
        .comp(TokenIter::from(input, Span::call_site()));
    let (fields, (updates, (predicates, (transactions, (deletions, name))))) =
        res.to_result().map_err(TokenDiagnostic::into_list)?;
    analyse(fields, updates, predicates, transactions, deletions, name)
}
