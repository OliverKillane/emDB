use std::collections::LinkedList;

use proc_macro2::{Delimiter, Ident, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use syn::{Expr, Type};

use super::ast::{
    Connector, Constraint, ConstraintExpr, FuncOp, Operator, Query, StreamExpr, Table, AST,
};
use crate::utils::parst::{
    core::{either, many0, mapsuc, recover, recursive, seq, ParseResult, Parser, RecursiveHandle},
    macros::{choice, seqs},
    tokens::{
        collectuntil, error, getident, gettoken, ingroup, isempty, listsep, matchident, matchpunct,
        not, nothing, peekident, peekpunct, recoverpunct, syn, syntopunct, SpannedCont,
        SpannedError, TokenIter,
    },
};

pub(super) fn parse(ts: TokenStream) -> Result<AST, LinkedList<Diagnostic>> {
    let parser = emdb_parser(); // todo

    let (_, res) = parser.parse(TokenIter::from(ts));

    match res {
        ParseResult::Suc(o) => Err(LinkedList::new()), // temporary
        ParseResult::Con(c) => Err(c.into_list()),
        ParseResult::Err(e) => Err(SpannedCont::from_err(e).into_list()),
    }
}

enum QueryTable {
    Query(Query),
    Table(Table),
}

fn emdb_parser() -> impl Parser<TokenIter, O = AST, C = SpannedCont, E = SpannedError> {
    mapsuc(
        seq(
            name_parser(),
            many0(
                not(isempty()),
                choice!(
                    peekident("query") => mapsuc(parse_query(), |q| QueryTable::Query(q)),
                    peekident("table") => mapsuc(parse_table(), |t| QueryTable::Table(t)),
                    otherwise => error(gettoken(), |t| {
                        Diagnostic::spanned(t.span(), Level::Error, String::from("expected query or table"))
                    })
                ),
            ),
        ),
        |(name, objects)| {
            let mut tables = vec![];
            let mut queries = vec![];
            for obj in objects {
                match obj {
                    QueryTable::Query(q) => queries.push(q),
                    QueryTable::Table(t) => tables.push(t),
                }
            }
            AST {
                name,
                tables,
                queries,
            }
        },
    )
}

fn name_parser() -> impl Parser<TokenIter, O = Ident, C = SpannedCont, E = SpannedError> {
    mapsuc(
        recover(
            seqs!(matchident("name"), getident(), matchpunct(';')),
            recoverpunct(';'),
        ),
        |(_, (name, _))| name,
    )
}

fn parse_query() -> impl Parser<TokenIter, O = Query, C = SpannedCont, E = SpannedError> {
    mapsuc(
        seqs!(
            matchident("query"),
            getident(),
            ingroup(Delimiter::Parenthesis, member_list_parser()),
            ingroup(Delimiter::Brace, many0(isempty(), stream_parser()))
        ),
        |(_, (name, (params, streams)))| Query {
            name,
            params,
            streams,
        },
    )
}

fn parse_table() -> impl Parser<TokenIter, O = Table, C = SpannedCont, E = SpannedError> {
    mapsuc(
        seqs!(
            matchident("table"),
            getident(),
            ingroup(Delimiter::Brace, member_list_parser()),
            either(
                peekpunct('@'),
                mapsuc(
                    seqs!(
                        matchpunct('@'),
                        listsep(',', constraint_parser()),
                        matchpunct(';')
                    ),
                    |(_, (cons, _))| cons
                ),
                mapsuc(nothing(), |()| vec![])
            )
        ),
        |(_, (name, (cols, cons)))| Table { name, cols, cons },
    )
}

fn member_list_parser(
) -> impl Parser<TokenIter, O = Vec<(Ident, Type)>, C = SpannedCont, E = SpannedError> {
    // bug: listsep
    listsep(
        ',',
        mapsuc(
            seqs!(getident(), matchpunct(':'), syntopunct::<syn::Type>(',')),
            |(m, (_, t))| (m, t),
        ),
    )
}

fn constraint_parser() -> impl Parser<TokenIter, O = Constraint, C = SpannedCont, E = SpannedError>
{
    fn inner(
        name: &'static str,
        p: impl Parser<TokenIter, O = ConstraintExpr, C = SpannedCont, E = SpannedError>,
    ) -> impl Parser<TokenIter, O = Constraint, C = SpannedCont, E = SpannedError> {
        mapsuc(
            seqs!(
                matchident(name),
                ingroup(Delimiter::Parenthesis, p),
                either(
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

    choice!(
        peekident("unique") => inner("unique", mapsuc(getident(), |i| ConstraintExpr::Unique{field:i})),
        peekident("pred") => inner("pred", mapsuc(syn(collectuntil(isempty())), |e| ConstraintExpr::Pred(e))),
        peekident("genpk") => inner("genpk", mapsuc(getident(), |i| ConstraintExpr::GenPK{field:i})),
        peekident("limit") => inner("limit", mapsuc(syn(collectuntil(isempty())), |e| ConstraintExpr::Limit{size:e})),
        otherwise => error(getident(), |i| Diagnostic::spanned(i.span(), Level::Error, format!("expected a constraint but got {}", i.to_string())))
    )
}

fn connector_parse() -> impl Parser<TokenIter, O = Connector, C = SpannedCont, E = SpannedError> {
    choice!(
        peekpunct('~') => mapsuc(seq(matchpunct('~'), matchpunct('>')), |(t1, _)| Connector{single: true, span: t1.span()}),
        peekpunct('|') => mapsuc(seq(matchpunct('|'), matchpunct('>')), |(t1, _)| Connector{single: false, span: t1.span()}),
        otherwise => error(seq(gettoken(), gettoken()), |(t1, t2)| Diagnostic::spanned(t1.span(), Level::Error, format!("expected either ~> or |> but got {}{}", t1.to_string(), t2.to_string())))
    )
}

fn operator_parse(
    r: RecursiveHandle<TokenIter, StreamExpr, SpannedError, SpannedCont>,
) -> impl Parser<TokenIter, O = Operator, C = SpannedCont, E = SpannedError> {
    fn inner(
        name: &'static str,
        p: impl Parser<TokenIter, O = FuncOp, C = SpannedCont, E = SpannedError>,
    ) -> impl Parser<TokenIter, O = Operator, C = SpannedCont, E = SpannedError> {
        mapsuc(
            seq(matchident(name), ingroup(Delimiter::Parenthesis, p)),
            |(id, op)| Operator::FuncOp {
                fn_span: id.span(),
                op,
            },
        )
    }

    choice!(
        peekident("ret") => mapsuc(matchident("ret"), |m| Operator::Ret { ret_span: m.span() }),
        peekident("ref") => mapsuc(seq(matchident("ref"), getident()), |(m, table_name)| Operator::Ref { ref_span: m.span(), table_name }),
        peekident("let") => mapsuc(seq(matchident("let"), getident()), |(m, var_name)| Operator::Let { let_span: m.span(), var_name }),
        peekident("use") => mapsuc(seq(matchident("use"), getident()), |(m, var_name)| Operator::Use { use_span: m.span(), var_name }),
        peekident("scan") => inner("scan", mapsuc(getident(), |table_name| FuncOp::Scan{table_name})),
        // todo: Add other ops
        otherwise => error(gettoken(), |t| Diagnostic::spanned(t.span(), Level::Error, format!("expected an operator but got {}", t.to_string())))
    )
}

fn stream_parser() -> impl Parser<TokenIter, O = StreamExpr, C = SpannedCont, E = SpannedError> {
    recursive(|r| {
        mapsuc(
            seq(
                operator_parse(r.clone()),
                either(
                    peekpunct(';'),
                    mapsuc(nothing(), |()| None),
                    mapsuc(seq(connector_parse(), r), |(c, s)| Some((c, Box::new(s)))),
                ),
            ),
            |(op, con)| StreamExpr { op, con },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
}
