use crate::frontend::{
    emql::ast::{Query, Table, AST},
    Diagnostics,
};
use proc_macro2::{
    token_stream::IntoIter, Delimiter, Group, Ident, Punct, Span, TokenStream, TokenTree,
};
use proc_macro_error::{Diagnostic, DiagnosticExt, Level, SpanRange};

use super::ast::{SingleType, Spanned};

/// Parses the provided [TokenStream] as emQL to produce errors (in [Diagnostics]), and if possible an [AST].
/// - Even if errors are present, if the [AST] can be produced, then it can be returned and analysed.
/// - Multiple syntax errors are supported by adding to [Diagnostics]
///
/// Rather than use a parser combinator this is hand written as:
/// - Tokentree abstraction + [IntoIter] being one-way + error recovery do not fit well to parser combinator abstractions
/// - emQL is relatively simple, complex parsing done by [syn] / rust syntax.
/// - Need best possible performance
pub(super) fn parse(ts: TokenStream, errs: &mut Diagnostics) -> Option<AST> {
    let mut iter = ts.into_iter();
    let name = parse_database_name(&mut iter, errs);
    let mut tables: Vec<Table> = vec![];
    let mut queries: Vec<Query> = vec![];

    loop {
        match parse_table_or_query(&mut iter, errs) {
            QueryOrTable::Query(q) => queries.push(q),
            QueryOrTable::Table(t) => tables.push(t),
            QueryOrTable::End => break,
            QueryOrTable::Err => (),
        }
    }

    if let Some((s, nm)) = name {
        Some(AST {
            name: Spanned { data: nm, span: s },
            tables,
            queries,
        });

        // for moment
        None
    } else {
        None
    }
}

fn recover_past(
    err: Diagnostic,
    start: TokenTree,
    stop_after: fn(&TokenTree) -> bool,
    iter: &mut IntoIter,
) -> Diagnostic {
    let start_span = start.span();
    let mut curr = start;
    while !stop_after(&curr) {
        if let Some(tt) = iter.next() {
            curr = tt;
        } else {
            break;
        }
    }
    err.span_range_note(
        SpanRange {
            first: start_span,
            last: curr.span(),
        },
        String::from("Skipped during parsing."),
    )
}

const fn match_punct<const P: char>() -> fn(&TokenTree) -> bool {
    |tt| {
        if let TokenTree::Punct(punct) = tt {
            punct.as_char() == P
        } else {
            false
        }
    }
}

fn parse_ident(
    iter: &mut IntoIter,
    errs: &mut Diagnostics,
    err_msg: fn(Option<&TokenTree>) -> String,
    recovery: fn(&TokenTree) -> bool,
) -> Option<Ident> {
    match iter.next() {
        Some(TokenTree::Ident(i)) => Some(i),
        Some(tt) => {
            errs.add(recover_past(
                Diagnostic::spanned(tt.span(), Level::Error, err_msg(Some(&tt))),
                tt,
                recovery,
                iter,
            ));
            None
        }
        None => {
            errs.add(Diagnostic::spanned(
                Span::call_site(),
                Level::Error,
                err_msg(None),
            ));
            None
        }
    }
}

fn parse_group(
    iter: &mut IntoIter,
    errs: &mut Diagnostics,
    delim: Delimiter,
    err_msg: fn(Option<&TokenTree>) -> String,
    recovery: fn(&TokenTree) -> bool,
) -> Option<Group> {
    match iter.next() {
        Some(TokenTree::Group(g)) if g.delimiter() == delim => Some(g),
        Some(tt) => {
            errs.add(recover_past(
                Diagnostic::spanned(tt.span(), Level::Error, err_msg(Some(&tt))),
                tt,
                recovery,
                iter,
            ));
            None
        }
        None => {
            errs.add(Diagnostic::spanned(
                Span::call_site(),
                Level::Error,
                err_msg(None),
            ));
            None
        }
    }
}

fn parse_punct(
    iter: &mut IntoIter,
    errs: &mut Diagnostics,
    sep: Option<char>,
    err_msg: fn(Option<&TokenTree>) -> String,
    recovery: fn(&TokenTree) -> bool,
) -> Option<Punct> {
    match iter.next() {
        Some(TokenTree::Punct(p)) => Some(p),
        Some(tt) => {
            errs.add(recover_past(
                Diagnostic::spanned(tt.span(), Level::Error, err_msg(Some(&tt))),
                tt,
                recovery,
                iter,
            ));
            None
        }
        None => {
            errs.add(Diagnostic::spanned(
                Span::call_site(),
                Level::Error,
                err_msg(None),
            ));
            None
        }
    }
}

/// Parses `name <dbname> ;`
/// - Recovers until the next semicolon
/// - errors on end of input.
fn parse_database_name(iter: &mut IntoIter, errs: &mut Diagnostics) -> Option<(Span, String)> {
    const NAME: &'static str = "name";
    const SEMICOLON: fn(&TokenTree) -> bool = match_punct::<';'>();

    let kw_name = parse_ident(iter, errs, |_| format!("Expected '{NAME}'"), SEMICOLON)?;

    if kw_name.to_string() != NAME {
        errs.add(recover_past(
            Diagnostic::spanned(
                kw_name.span(),
                Level::Error,
                format!("Expected '{NAME}', found '{kw_name}'"),
            ),
            TokenTree::Ident(kw_name),
            SEMICOLON,
            iter,
        ));
        return None;
    }

    let db_name = parse_ident(
        iter,
        errs,
        |_| format!("Expected the database name"),
        SEMICOLON,
    )?;

    let end = parse_punct(iter, errs, ';', |_| format!("Expected ';'"), SEMICOLON)?;

    Some((db_name.span(), db_name.to_string()))
}

enum QueryOrTable {
    Query(Query),
    Table(Table),
    Err,
    End,
}

fn parse_columns(
    tks: TokenStream,
    errs: &mut Diagnostics,
) -> Vec<(Spanned<String>, Spanned<SingleType>)> {
    todo!()
}

fn parse_table(iter: &mut IntoIter, errs: &mut Diagnostics) -> Option<Table> {
    let tb_name = parse_ident(
        iter,
        errs,
        |_| format!("Expected the table name"),
        match_punct::<';'>(),
    )?;

    let table_content = parse_group(
        iter,
        errs,
        Delimiter::Brace,
        |_| String::from("Expected a table definition '{' ... '}'"),
        match_punct::<';'>(),
    )?;

    let cols = parse_columns(table_content.stream(), errs);

    match_punct(
        iter,
        errs,
        '@',
        |_| format!("Expected '@'"),
        match_punct::<';'>(),
    );

    todo!()
}

fn parse_table_or_query(iter: &mut IntoIter, errs: &mut Diagnostics) -> QueryOrTable {
    const QUERY: &'static str = "query";
    const TABLE: &'static str = "table";

    let kw_query_table = match iter.next() {
        Some(TokenTree::Ident(i)) => i,
        Some(tt) => {
            errs.add(recover_past(
                Diagnostic::spanned(
                    tt.span(),
                    Level::Error,
                    format!("Expected either '{QUERY}' or '{TABLE}' definitions"),
                ),
                tt,
                match_punct::<';'>(),
                iter,
            ));
            return QueryOrTable::Err;
        }
        None => {
            return QueryOrTable::End;
        }
    };

    match kw_query_table.to_string().as_ref() {
        QUERY => QueryOrTable::End, // TODO
        TABLE => QueryOrTable::End, // TODO
        other => {
            errs.add(recover_past(
                Diagnostic::spanned(
                    kw_query_table.span(),
                    Level::Error,
                    format!("Expected either '{QUERY}' or '{TABLE}' definitions, found '{other}'"),
                ),
                TokenTree::Ident(kw_query_table),
                match_punct::<';'>(),
                iter,
            ));
            QueryOrTable::Err
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn can_parse_db_name() {
        fn check_names(ts: TokenStream, name: &str) {
            let mut errs = Diagnostics::new();
            let raw_name = parse_database_name(&mut ts.into_iter(), &mut errs);
            assert!(errs.empty());
            if let Some((_, parsed_name)) = raw_name {
                assert_eq!(parsed_name, name);
            } else {
                assert!(false)
            }
        }

        check_names(quote!(name bob;), "bob");
        check_names(quote!(name b;), "b");
    }

    #[test]
    fn recovery_from_invalid_names() {
        fn iter_get_name(ts: &mut IntoIter) -> Option<String> {
            let mut errs = Diagnostics::new();

            parse_database_name(ts, &mut errs).map_or(
                {
                    // assert!(!errs.empty());
                    None
                },
                |(_, n)| {
                    assert!(errs.empty());
                    Some(n)
                },
            )
        }

        let mut iter = quote! {
            name d-c;
            name bob;
            name ;
            na name;
            name hey;
        }
        .into_iter();

        assert_eq!(iter_get_name(&mut iter), None);
        assert_eq!(iter_get_name(&mut iter), Some(String::from("bob")));
        assert_eq!(iter_get_name(&mut iter), None);
        assert_eq!(iter_get_name(&mut iter), None);
        assert_eq!(iter_get_name(&mut iter), Some(String::from("hey")));
    }
}
