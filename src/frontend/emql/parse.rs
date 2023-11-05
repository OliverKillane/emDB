use crate::frontend::emql::ast::{Query, Table, AST};
use proc_macro2::{token_stream::IntoIter, Ident, TokenStream, TokenTree};
use proc_macro_error::{Diagnostic, Level};

struct Diagnostics {
    errs: Vec<Diagnostic>,
}

impl Diagnostics {
    fn new() -> Self {
        Self { errs: Vec::new() }
    }
    fn add(&mut self, d: Diagnostic) {
        self.errs.push(d)
    }
    fn emit(self) {
        self.errs.into_iter().for_each(Diagnostic::emit)
    }
}

pub(super) fn parse(ts: TokenStream) -> Result<AST, Vec<Diagnostic>> {
    todo!()
}

enum TableQueryResult {
    Table(Table),
    Query(Query),
    End,
    Err,
}

fn parse_table_or_query(iter: &mut IntoIter, errs: &mut Diagnostics) -> TableQueryResult {
    fn recover(mut err: Diagnostic, mut curr: TokenTree, iter: &mut IntoIter) -> Diagnostic {
        loop {
            if let TokenTree::Group(_) = curr {
                break err;
            }
            err = err.span_error(curr.span(), String::from("Ignored due to previous error"));
            if let Some(tt) = iter.next() {
                curr = tt;
            } else {
                break err;
            }
        }
    }

    const QUERY: &'static str = "query";
    const TABLE: &'static str = "table";

    match iter.next() {
        Some(TokenTree::Ident(i)) if i.to_string() == QUERY => match iter.next() {
            Some(TokenTree::Ident(i)) => {}
            Some(tt) => {
                errs.add(recover(
                    Diagnostic::spanned(
                        tt.span(),
                        Level::Error,
                        format!("Expected the name of a {TABLE}"),
                    ),
                    tt,
                    iter,
                ));
                TableQueryResult::Err
            }
            None => {
                errs.add(Diagnostic::spanned(
                    i.span(), // TODO: opt into extra proc_macro2 features to add Span::end(&self) -> Span
                    Level::Error,
                    format!("Expected the name for a {TABLE} to follow."),
                ));
                TableQueryResult::Err
            }
        },
        Some(TokenTree::Ident(i)) if i.to_string() == TABLE => {
            todo!()
        }
        Some(tt) => {
            errs.add(recover(
                Diagnostic::spanned(
                    tt.span(),
                    Level::Error,
                    format!("Expected either '{QUERY}' or '{TABLE}'"),
                ),
                tt,
                iter,
            ));
            TableQueryResult::Err
        }
        None => TableQueryResult::End,
    }
}

enum TableResult {
    Ok(Table),
    End,
    Err,
}

fn parse_table(iter: &mut IntoIter, errs: &mut Diagnostics) -> TableResult {
    todo!()
}
