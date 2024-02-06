use std::collections::LinkedList;

use proc_macro2::{Delimiter, Ident, Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use syn::{Expr, Type};

use super::ast::{
    Ast, BackendImpl, Connector, Constraint, ConstraintExpr, FuncOp, Operator, Query, StreamExpr,
    Table,
};
use crate::frontend::emql::ast::SortOrder;

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
        error::{embelisherr, error, expectederr},
        recovery::until,
        TokenDiagnostic, TokenIter, TokenParser,
    },
    Combi, CombiResult,
};

pub(super) fn parse(ts: TokenStream) -> Result<Ast, LinkedList<Diagnostic>> {
    let parser = emql_parser();
    let (_, res) =
        mapsuc(seqdiff(parser, terminal), |(o, ())| o).comp(TokenIter::from(ts, Span::call_site()));

    match res {
        CombiResult::Suc(s) => Ok(s),
        CombiResult::Con(es) | CombiResult::Err(es) => Err(es.into_list()),
    }
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
                seqs!(matchident("impl"), getident(), matchident("as"), getident()),
                until(peekpunct(';')),
            ),
            matchpunct(';'),
        ),
        |((_, (db_backend, (_, db_name))), _)| BackendImpl {
            name: db_name,
            target: db_backend,
        },
    )
}

fn query_parser() -> impl TokenParser<Query> {
    expectederr(mapsuc(
        seqs!(
            matchident("query"),
            getident(),
            recovgroup(Delimiter::Parenthesis, member_list_parser()),
            recovgroup(Delimiter::Brace, many0(not(isempty()), stream_parser()))
        ),
        |(_, (name, (params, streams)))| Query {
            name,
            params,
            streams,
        },
    ))
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
    setrepr(
        listseptrailing(
            ',',
            mapsuc(
                seqs!(getident(), matchpunct(':'), syntopunct(peekpunct(','))),
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
                    mapsuc(nothing(), |_| None)
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
        peekident("genpk") => inner("genpk", mapsuc(getident(), |i| ConstraintExpr::GenPK{field:i})),
        peekident("limit") => inner("limit", mapsuc(syn(collectuntil(isempty())), |e| ConstraintExpr::Limit{size:e})),
        otherwise => error(getident(), |i| Diagnostic::spanned(i.span(), Level::Error, format!("expected a constraint but got {}", i)))
    )
}

fn connector_parse() -> impl TokenParser<Connector> {
    embelisherr(
        choices!(
            peekpunct('~') => mapsuc(seq(matchpunct('~'), matchpunct('>')), |(t1, _)| Connector{single: true, span: t1.span()}),
            peekpunct('|') => mapsuc(seq(matchpunct('|'), matchpunct('>')), |(t1, _)| Connector{single: false, span: t1.span()}),
            otherwise => error(seq(gettoken, gettoken), |(t1, t2)| Diagnostic::spanned(t1.span(), Level::Error, format!("expected either ~> or |> but got {}{}", t1, t2)))
        ),
        "Connect operators a single row passed (`~>`), or a stream of rows (`|>`)",
    )
}

fn operator_parse(
    r: RecursiveHandle<TokenIter, TokenIter, StreamExpr, TokenDiagnostic, TokenDiagnostic>,
) -> impl TokenParser<Operator> {
    fn inner(name: &'static str, p: impl TokenParser<FuncOp>) -> impl TokenParser<Operator> {
        mapsuc(
            seq(matchident(name), recovgroup(Delimiter::Parenthesis, p)),
            |(id, op)| Operator::FuncOp {
                fn_span: id.span(),
                op,
            },
        )
    }

    fn fields_expr() -> impl TokenParser<Vec<(Ident, Expr)>> {
        listseptrailing(
            ',',
            mapsuc(
                seqs!(getident(), matchpunct('='), syntopunct(peekpunct(','))),
                |(id, (_, exp))| (id, exp),
            ),
        )
    }

    fn fields_assign() -> impl TokenParser<Vec<(Ident, Type, Expr)>> {
        listseptrailing(
            ',',
            mapsuc(
                seqs!(
                    getident(),
                    matchpunct(':'),
                    syntopunct(peekpunct('=')),
                    matchpunct('='),
                    syntopunct(peekpunct(','))
                ),
                |(id, (_, (t, (_, e))))| (id, t, e),
            ),
        )
    }

    choices!(
        peekident("return") => mapsuc(matchident("return"), |m| Operator::Ret { ret_span: m.span() }),
        peekident("ref") => mapsuc(seq(matchident("ref"), getident()), |(m, table_name)| Operator::Ref { ref_span: m.span(), table_name }),
        peekident("let") => mapsuc(seq(matchident("let"), getident()), |(m, var_name)| Operator::Let { let_span: m.span(), var_name }),
        peekident("use") => mapsuc(seq(matchident("use"), getident()), |(m, var_name)| Operator::Use { use_span: m.span(), var_name }),
        peekident("update") => inner("update", mapsuc(seqs!(getident(), matchident("use"), fields_expr()), |(reference, (_, fields))| FuncOp::Update {reference, fields})),
        peekident("insert") => inner("insert", mapsuc(getident(), |table_name| FuncOp::Insert{table_name})),
        peekident("delete") => inner("delete", mapsuc(nothing(), |()| FuncOp::Delete)),
        peekident("map") => inner("map", mapsuc(fields_assign(), |new_fields| FuncOp::Map{new_fields})),
        peekident("unique") => inner("unique", mapsuc(seqs!(matchident("use"), getident(), matchident("as"), getident()), |(_, (from_field, (_, unique_field)))|  FuncOp::Unique { unique_field, from_field } )),
        peekident("filter") => inner("filter", mapsuc(syn(collectuntil(isempty())), FuncOp::Filter)),
        peekident("row") => inner("row", mapsuc(fields_assign(), |fields| FuncOp::Row{fields})),
        peekident("sort") => inner("sort", mapsuc(listseptrailing(',', mapsuc(seq(getident(), choices!(
            peekident("asc") => mapsuc(matchident("asc"), |t| (SortOrder::Asc, t.span())),
            peekident("desc") => mapsuc(matchident("desc"), |t| (SortOrder::Desc, t.span())),
            otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("Can only sort by `asc` or `desc`, not by {:?}", t)))
        )), |(i, (o, s))| (i, o, s))), |fields| FuncOp::Sort{fields})),
        peekident("fold") => inner("fold", mapsuc(seqs!(
            recovgroup(Delimiter::Parenthesis, fields_assign()),
            matchpunct('='),
            matchpunct('>'),
            recovgroup(Delimiter::Parenthesis, fields_expr())
        ) , |(initial, (_, (_, update)))| FuncOp::Fold {initial, update})),
        peekident("assert") => inner("assert", mapsuc(syn(collectuntil(isempty())), FuncOp::Assert)),
        peekident("collect") => inner("collect", mapsuc(nothing(), |()| FuncOp::Collect)),
        otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("expected an operator but got {}", t)))
    )
}

fn stream_parser() -> impl TokenParser<StreamExpr> {
    recursive(|r| {
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
    })
}
