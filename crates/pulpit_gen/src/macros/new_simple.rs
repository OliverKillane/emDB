
#![allow(dead_code, unused_variables, unused_imports)]
//! # A simple interface for generating tables.
//! TODO: improve this using the new [`combi::tokens::options`] parser
use std::collections::{HashMap, HashSet, LinkedList};

use combi::{
    core::{choice, mapall, mapsuc, nothing, recover, seq, seqdiff},
    logical::or,
    macros::{choices, seqs},
    tokens::{
        basic::{
            collectuntil, getident, gettoken, isempty, matchident, matchpunct, peekident,
            peekpunct, recovgroup, terminal,
        }, derived::listseptrailing, error::{error, expectederr}, options::{DefaultField, MustField, OptEnd, OptField, OptParse}, recovery::until, TokenDiagnostic, TokenIter, TokenParser
    },
    Combi, CombiResult,
};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use syn::Ident;

use crate::{
    groups::Field,
    limit::{Limit, LimitKind},
    operations::{get::Get, update::Update},
    predicates::Predicate,
    selector::SelectOperations,
    uniques::Unique,
};

struct Access {
    alias: Ident,
    fields: Vec<Ident>
}

pub fn on_off() -> impl TokenParser<bool> {
    choices!(
        peekident("on") => mapsuc(matchident("on"), |_| true),
        peekident("off") => mapsuc(matchident("off"), |_| false),
        otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, "Expected `on` or `off`".to_owned()))
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
    expectederr(
        recovgroup(proc_macro2::Delimiter::Brace, inner),
    )
}

fn parse_access() -> impl TokenParser<Vec<Access>> {
    let fields_parser = mapsuc(
        seqs!(
            getident(),
            matchpunct(':'),
            recovgroup(
                proc_macro2::Delimiter::Bracket,
                listseptrailing(',', getident())
            )
        ),
        |(alias, (_, fields))| Access { alias, fields },
    );
    recovgroup(
        proc_macro2::Delimiter::Brace,
        listseptrailing(',', fields_parser),
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
    recovgroup(
        proc_macro2::Delimiter::Brace,
        listseptrailing(',', predicate_parser),
    )
}

fn limit_parser() -> impl TokenParser<Limit> {
    recovgroup(
        proc_macro2::Delimiter::Brace,
        mapsuc(
                seqs!(
                    getident(),
                    matchpunct(':'),
                    collectuntil(isempty())
                ),
                |( alias, (_, tks))| Limit { value: LimitKind::ConstVal(tks.into()), alias })
    )
}

struct ASTField {
    field_kind: Field,
    unique: Option<Ident>,
}


fn analyse(
    fields: Vec<ASTField>,
    updates: Vec<Access>,
    gets: Vec<Access>,
    predicates: Vec<Predicate>,
    limit: Option<Limit>,
    transactions: bool,
    deletions: bool,
    name: Ident,
    public: bool,
) -> Result<SelectOperations, TokenDiagnostic> {
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

    if let Some(Limit { alias, .. }) = &limit {
        if let Some(prev_name) = seen_access_names.get(alias) {
            add_duplicate(alias, prev_name)
        } else {
            seen_access_names.insert(alias.clone());
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

    if errors.is_empty() {
        Ok(SelectOperations {
            name,
            transactions,
            deletions,
            fields: field_types,
            uniques,
            predicates,
            gets: gets.into_iter().map(|Access { alias, fields }| Get { alias, fields }).collect(),
            updates: updates.into_iter().map(|Access { alias, fields }| Update { alias, fields }).collect(),
            public,
            limit,
        })
    } else {
        Err(TokenDiagnostic::from_list(errors).unwrap()) // at least one error! (not empty)
    }

}



// BUG!: Unfortunately this interface tyoe checks (`cargo check`) but crashes the rust compiler (I think out of resources)
// TODO: determine why this nukes the compiler (I suspect too much impl trait)

// fn parse() -> impl TokenParser<SelectOperations> {
//     mapall((MustField::new("name", getident),
//         (
//             DefaultField::new("transactions", on_off, ||false),
//             (
//                 DefaultField::new("deletions", on_off, ||false),
//                 (
//                     MustField::new("fields", fields_parser),
//                     (
//                         DefaultField::new("gets", parse_access, ||Vec::new()),
//                         (
//                             DefaultField::new("updates", parse_access,|| Vec::new()),
//                             (
//                                 DefaultField::new("predicates", parse_predicates,|| Vec::new()),
//                                 (
//                                     OptField::new("limit", limit_parser),
//                                     (
//                                         DefaultField::new("public", on_off, ||false),
//                                         OptEnd
//                                     )
//                                 )
//                             )
//                         )
//                     )
//                 )
//             )
//         )
//     ).gen(':'), 
//     |(name, (transactions, (deletions, (fields, (gets, (updates, (predicates, (limit, (public, ())))))))))| {
//         match analyse(fields, updates, gets, predicates, limit, transactions, deletions, name, public) {
//             Ok(s) => CombiResult::Suc(s),
//             Err(e) => CombiResult::Con(e),
//         }
//     })
// } 

pub fn simple(tks: TokenStream) -> Result<SelectOperations, LinkedList<Diagnostic>> {
    // parse().comp(TokenIter::from(tks, Span::call_site())).1.to_result().map_err(TokenDiagnostic::into_list)
    unimplemented!()
}
