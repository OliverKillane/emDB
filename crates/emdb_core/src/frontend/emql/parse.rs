//! # Parsing
//! The main parser for the emql language
//! - Operator parsers are defined in [`super::operators`]
//! - Common parsing utilities are defined here
//! - The main [`emql_parser`] to invoke for emql through [`parse`].
//!
//! ## Design
//! Language design is discussed in [emql](super).
//!
//! These parsers rely on 1 token lookahead, this results in the language having
//! - heavy use of initialiser keywords
//! - uncomplex/simple grammar, with no ambiguity
//!
//! ## Potential Improvements
//! ### Parallel Parsing
//! Relying on changes to [combi] and the `!Send + !Sync` properties of [`TokenStream`].

use super::{ast, operators::parse_operator};
use combi::{
    core::{choice, mapsuc, nothing, recover, recursive, seq, seqdiff, setrepr, RecursiveHandle},
    derived::many0,
    logical::{not, or},
    macros::{choices, seqs},
    tokens::{
        basic::{
            collectuntil, getident, gettoken, isempty, matchident, matchpunct, peekident,
            peekpunct, recovgroup, syn, terminal,
        },
        derived::{listseptrailing, syntopunct},
        error::{embelisherr, error},
        recovery::until,
        TokenDiagnostic, TokenIter, TokenParser,
    },
    Combi,
};
use proc_macro2::{Delimiter, Ident, Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use std::collections::LinkedList;
use syn::{Expr, Type};

pub(super) fn parse(ts: TokenStream) -> Result<ast::Ast, LinkedList<Diagnostic>> {
    let parser = emql_parser();
    let (_, res) =
        mapsuc(seqdiff(parser, terminal), |(o, ())| o).comp(TokenIter::from(ts, Span::call_site()));
    res.to_result().map_err(TokenDiagnostic::into_list)
}

enum EmqlItem {
    Query(ast::Query),
    Table(ast::Table),
    Backend(ast::BackendImpl),
}

fn emql_parser() -> impl TokenParser<ast::Ast> {
    mapsuc(
        many0(
            not(isempty()),
            recover(
                choices!(
                    peekident("query") => mapsuc(query_parser(), EmqlItem::Query),
                    peekident("table") => mapsuc(table_parser(), EmqlItem::Table),
                    peekident("impl") => mapsuc(backend_parser(), EmqlItem::Backend),
                    otherwise => error(gettoken, |t| {
                        Diagnostic::spanned(t.span(), Level::Error, String::from("expected impl, query or table"))
                    })
                ),
                until(or(
                    peekident("table"),
                    or(peekident("query"), peekident("impl")),
                )),
            ),
        ),
        |emql_items| {
            let mut tables = vec![];
            let mut queries = vec![];
            let mut backends = vec![];
            for obj in emql_items {
                match obj {
                    EmqlItem::Query(q) => queries.push(q),
                    EmqlItem::Table(t) => tables.push(t),
                    EmqlItem::Backend(b) => backends.push(b),
                }
            }
            ast::Ast {
                backends,
                tables,
                queries,
            }
        },
    )
}

fn backend_parser() -> impl TokenParser<ast::BackendImpl> {
    mapsuc(
        seq(
            recover(
                seqs!(
                    matchident("impl"),
                    getident(),
                    matchident("as"),
                    getident(),
                    choices!(
                        peekpunct(';') => mapsuc(nothing(), |()| None),
                        otherwise => mapsuc(recovgroup(Delimiter::Brace, collectuntil(isempty())), Option::Some)
                    )
                ),
                until(peekpunct(';')),
            ),
            matchpunct(';'),
        ),
        |((_, (impl_name, (_, (backend_name, options)))), _)| ast::BackendImpl {
            impl_name,
            backend_name,
            options,
        },
    )
}

pub type ContextRecurHandle =
    RecursiveHandle<TokenIter, TokenIter, Vec<ast::StreamExpr>, TokenDiagnostic, TokenDiagnostic>;

fn context_parser() -> impl TokenParser<Vec<ast::StreamExpr>> {
    recursive(|context_recur: ContextRecurHandle| {
        many0(
            not(isempty()),
            recover(
                recursive(|operator_recur| {
                    mapsuc(
                        seq(
                            parse_operator(context_recur.clone()),
                            choice(
                                peekpunct(';'),
                                mapsuc(matchpunct(';'), |_| None),
                                mapsuc(seq(connector_parse(), operator_recur), |(c, s)| {
                                    Some((c, Box::new(s)))
                                }),
                            ),
                        ),
                        |(op, con)| ast::StreamExpr { op, con },
                    )
                }),
                until(choice(
                    peekpunct(';'),
                    mapsuc(matchpunct(';'), |_| true),
                    mapsuc(nothing(), |()| false),
                )),
            ),
        )
    })
}

fn query_parser() -> impl TokenParser<ast::Query> {
    mapsuc(
        seqs!(
            matchident("query"),
            getident(),
            recovgroup(Delimiter::Parenthesis, query_param_list_parser()),
            recovgroup(Delimiter::Brace, context_parser())
        ),
        |(_, (name, (params, streams)))| ast::Query {
            name,
            context: ast::Context { params, streams },
        },
    )
}

fn table_parser() -> impl TokenParser<ast::Table> {
    mapsuc(
        seqs!(
            matchident("table"),
            getident(),
            recovgroup(Delimiter::Brace, member_list_parser()),
            choice(
                peekpunct('@'),
                mapsuc(
                    seq(
                        matchpunct('@'),
                        recovgroup(
                            Delimiter::Bracket,
                            listseptrailing(',', constraint_parser())
                        )
                    ),
                    |(_, cons)| cons
                ),
                mapsuc(nothing(), |()| vec![])
            )
        ),
        |(_, (name, (cols, cons)))| ast::Table { name, cols, cons },
    )
}

fn member_list_parser() -> impl TokenParser<Vec<(Ident, Type)>> {
    listseptrailing(
        ',',
        mapsuc(
            seqs!(getident(), matchpunct(':'), syntopunct(peekpunct(','))),
            |(m, (_, t))| (m, t),
        ),
    )
}

fn query_param_list_parser() -> impl TokenParser<Vec<(Ident, ast::AstType)>> {
    setrepr(
        listseptrailing(
            ',',
            mapsuc(
                seqs!(getident(), matchpunct(':'), type_parser_to_punct(',')),
                |(m, (_, t))| (m, t),
            ),
        ),
        "<name> : <Type>, ...",
    )
}

fn constraint_parser() -> impl TokenParser<ast::Constraint> {
    fn inner(
        name: &'static str,
        p: impl TokenParser<ast::ConstraintExpr>,
    ) -> impl TokenParser<ast::Constraint> {
        mapsuc(
            seqs!(
                matchident(name),
                recovgroup(Delimiter::Parenthesis, p),
                matchident("as"),
                getident()
            ),
            |(method, (p, (_, alias)))| ast::Constraint {
                alias,
                method_span: method.span(),
                expr: p,
            },
        )
    }

    choices!(
        peekident("unique") => inner("unique", mapsuc(getident(), |i| ast::ConstraintExpr::Unique{field:i})),
        peekident("pred") => inner("pred", mapsuc(syn(collectuntil(isempty())), ast::ConstraintExpr::Pred)),
        peekident("limit") => inner("limit", mapsuc(syn(collectuntil(isempty())), |e| ast::ConstraintExpr::Limit{size:e})),
        otherwise => error(getident(), |i| Diagnostic::spanned(i.span(), Level::Error, format!("expected a constraint (e.g. pred, unique) but got {i}")))
    )
}

fn connector_parse() -> impl TokenParser<ast::Connector> {
    embelisherr(
        choices!(
            peekpunct('~') => mapsuc(
                seq(
                    matchpunct('~'),
                    matchpunct('>')
                ),
                |(t1, _)| ast::Connector{stream: false, span: t1.span()}
            ),
            peekpunct('|') => mapsuc(
                seq(
                    matchpunct('|'),
                    matchpunct('>')
                ),
                |(t1, _)| ast::Connector{stream: true, span: t1.span()}
            ),
            otherwise => error(
                seq(
                    gettoken,
                    gettoken
                ),
                |(t1, t2)| Diagnostic::spanned(t1.span(), Level::Error, format!("expected either ~> or |> but got {t1}{t2}"))
            )
        ),
        "Connect operators a single row passed (`~>`), or a stream of rows (`|>`)",
    )
}

pub fn type_parser(end: impl TokenParser<bool>) -> impl TokenParser<ast::AstType> {
    setrepr(
        choices! {
            peekident("ref") => mapsuc(seq(matchident("ref"), getident()), |(_, i)| ast::AstType::TableRef(i)),
            peekident("type") => mapsuc(seq(matchident("type"), getident()), |(_, i)| ast::AstType::Custom(i)),
            otherwise => mapsuc(syntopunct(end), ast::AstType::RsType)
        },
        "<type>",
    )
}

pub fn type_parser_to_punct(end: char) -> impl TokenParser<ast::AstType> {
    type_parser(peekpunct(end))
}

// helper function for the functional style operators
pub fn functional_style<T>(
    name: &'static str,
    p: impl TokenParser<T>,
) -> impl TokenParser<(Ident, T)> {
    seq(matchident(name), recovgroup(Delimiter::Parenthesis, p))
}

pub fn fields_expr() -> impl TokenParser<Vec<(Ident, Expr)>> {
    listseptrailing(
        ',',
        mapsuc(
            seqs!(getident(), matchpunct('='), syntopunct(peekpunct(','))),
            |(id, (_, exp))| (id, exp),
        ),
    )
}

pub fn fields_assign() -> impl TokenParser<Vec<(Ident, (ast::AstType, Expr))>> {
    listseptrailing(
        ',',
        mapsuc(
            seqs!(
                setrepr(getident(), "<field name>"),
                matchpunct(':'),
                type_parser_to_punct('='),
                matchpunct('='),
                setrepr(syntopunct(peekpunct(',')), "<expression>")
            ),
            |(id, (_, (t, (_, e))))| (id, (t, e)),
        ),
    )
}
