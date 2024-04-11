use std::collections::LinkedList;

use proc_macro2::{Delimiter, Ident, Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use syn::{Expr, Type};

use super::{
    ast::{
        Ast, AstType, BackendImpl, Connector, Constraint, ConstraintExpr, Query, StreamExpr, Table,
    },
    operators::{parse_operator, Operator},
};

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

pub(super) fn parse(ts: TokenStream) -> Result<Ast, LinkedList<Diagnostic>> {
    let parser = emql_parser();
    let (_, res) =
        mapsuc(seqdiff(parser, terminal), |(o, ())| o).comp(TokenIter::from(ts, Span::call_site()));
    res.to_result().map_err(TokenDiagnostic::into_list)
}

enum EmqlItem {
    Query(Query),
    Table(Table),
    Backend(BackendImpl),
}

fn emql_parser() -> impl TokenParser<Ast> {
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
            Ast {
                backends,
                tables,
                queries,
            }
        },
    )
}

fn backend_parser() -> impl TokenParser<BackendImpl> {
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
        |((_, (impl_name, (_, (backend_name, options)))), _)| BackendImpl {
            impl_name,
            backend_name,
            options,
        },
    )
}

fn query_parser() -> impl TokenParser<Query> {
    mapsuc(
        seqs!(
            matchident("query"),
            getident(),
            recovgroup(Delimiter::Parenthesis, query_param_list_parser()),
            recovgroup(Delimiter::Brace, many0(not(isempty()), stream_parser()))
        ),
        |(_, (name, (params, streams)))| Query {
            name,
            params,
            streams,
        },
    )
}

fn table_parser() -> impl TokenParser<Table> {
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
        |(_, (name, (cols, cons)))| Table { name, cols, cons },
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

fn query_param_list_parser() -> impl TokenParser<Vec<(Ident, AstType)>> {
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

fn constraint_parser() -> impl TokenParser<Constraint> {
    fn inner(
        name: &'static str,
        p: impl TokenParser<ConstraintExpr>,
    ) -> impl TokenParser<Constraint> {
        mapsuc(
            seqs!(
                matchident(name),
                recovgroup(Delimiter::Parenthesis, p),
                choice(
                    peekident("as"),
                    mapsuc(seq(matchident("as"), getident()), |(_, i)| Some(i)),
                    mapsuc(nothing(), |()| None)
                )
            ),
            |(method, (p, alias))| Constraint {
                alias,
                method_span: method.span(),
                expr: p,
            },
        )
    }

    choices!(
        peekident("unique") => inner("unique", mapsuc(getident(), |i| ConstraintExpr::Unique{field:i})),
        peekident("pred") => inner("pred", mapsuc(syn(collectuntil(isempty())), ConstraintExpr::Pred)),
        peekident("limit") => inner("limit", mapsuc(syn(collectuntil(isempty())), |e| ConstraintExpr::Limit{size:e})),
        otherwise => error(getident(), |i| Diagnostic::spanned(i.span(), Level::Error, format!("expected a constraint (e.g. pred, unique) but got {i}")))
    )
}

fn connector_parse() -> impl TokenParser<Connector> {
    embelisherr(
        choices!(
            peekpunct('~') => mapsuc(
                seq(
                    matchpunct('~'),
                    matchpunct('>')
                ),
                |(t1, _)| Connector{stream: false, span: t1.span()}
            ),
            peekpunct('|') => mapsuc(
                seq(
                    matchpunct('|'),
                    matchpunct('>')
                ),
                |(t1, _)| Connector{stream: true, span: t1.span()}
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

type RecursiveExpr =
    RecursiveHandle<TokenIter, TokenIter, StreamExpr, TokenDiagnostic, TokenDiagnostic>;

fn operator_parse(
    r: RecursiveExpr, // TODO: make operators recursive (for groupby and join)
) -> impl TokenParser<Operator> {
    parse_operator()
}

fn stream_parser() -> impl TokenParser<StreamExpr> {
    recover(recursive(|r| {
        mapsuc(
            seq(
                operator_parse(r.clone()),
                choice(
                    peekpunct(';'),
                    mapsuc(matchpunct(';'), |_| None),
                    mapsuc(seq(connector_parse(), r), |(c, s)| Some((c, Box::new(s)))),
                ),
            ),
            |(op, con)| StreamExpr { op, con },
        )
    }), until(choice(peekpunct(';'), mapsuc(matchpunct(';'), |_| true), mapsuc(nothing(), |()| false))))
}

pub fn type_parser(end: impl TokenParser<bool>) -> impl TokenParser<AstType> {
    setrepr(
        choices! {
            peekident("ref") => mapsuc(seq(matchident("ref"), getident()), |(_, i)| AstType::TableRef(i)),
            peekident("type") => mapsuc(seq(matchident("type"), getident()), |(_, i)| AstType::Custom(i)),
            otherwise => mapsuc(syntopunct(end), AstType::RsType)
        },
        "<type>",
    )
}

pub fn type_parser_to_punct(end: char) -> impl TokenParser<AstType> {
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

pub fn fields_assign() -> impl TokenParser<Vec<(Ident, (AstType, Expr))>> {
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
