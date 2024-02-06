use std::collections::{HashMap, HashSet, LinkedList};

use proc_macro2::Ident;
use proc_macro_error::{Diagnostic, Level};
use syn::spanned::Spanned;

use crate::{
    frontend::emql::ast::{Ast, ConstraintExpr},
    plan::{
        repr::{
            LogicalColumn, LogicalColumnConstraint, LogicalPlan, LogicalQuery,
            LogicalRowConstraint, LogicalTable, UniqueCons,
        },
        targets::{Target, Targets},
    },
};

use super::ast::{BackendImpl, Constraint, Query, Table};

pub(super) fn translate(
    Ast {
        backends,
        tables,
        queries,
    }: Ast,
) -> Result<(Targets, LogicalPlan), LinkedList<Diagnostic>> {
    let mut logical_plan = LogicalPlan::new();

    let mut errs = LinkedList::new();

    for table in tables {
        match translate_table(table) {
            Ok(table) => {
                logical_plan.tables.insert(table);
            }
            Err(mut table_errs) => {
                errs.append(&mut table_errs);
            }
        }
    }

    match extract_targets(backends) {
        Ok(ts) => {
            if errs.len() > 0 {
                Err(errs)
            } else {
                Ok((ts, logical_plan))
            }
        }
        Err(mut es) => {
            errs.append(&mut es);
            Err(errs)
        }
    }
}

/// Extract the targets/backends to be run for the plan
pub(super) fn extract_targets(
    parsed_tagets: Vec<BackendImpl>,
) -> Result<Targets, LinkedList<Diagnostic>> {
    let mut backends: HashMap<Ident, Target> = HashMap::new();
    let mut errs = LinkedList::new();

    for BackendImpl { name, target } in parsed_tagets {
        if let Some((duplicate, _)) = backends.get_key_value(&name) {
            errs.push_back(
                Diagnostic::spanned(
                    name.span(),
                    Level::Error,
                    format!(
                        "Cannot use name `{}` as it is already used. Names must be unique",
                        name
                    ),
                )
                .span_note(duplicate.span(), "Already used here".to_owned()),
            );
        } else {
            let target = match target.to_string().as_str() {
                "graph" => Some(Target::Graphviz),
                "simple" => Some(Target::Simple),
                _ => {
                    errs.push_back(Diagnostic::spanned(
                        target.span(),
                        Level::Error,
                        format!("Unknown target `{}`", target),
                    ));
                    None
                }
            };

            if let Some(target) = target {
                backends.insert(name, target);
            }
        }
    }

    if errs.len() > 0 {
        Err(errs)
    } else {
        Ok(Targets { backends })
    }
}

/// Translate a table definition into the logical plan
///
/// Must ensure the following invariants hold:
/// - Each column has a unique name
/// - Each constraint has a unique name
pub(super) fn translate_table(
    Table { name, cols, cons }: Table,
) -> Result<LogicalTable, LinkedList<Diagnostic>> {
    let mut errs = LinkedList::new();

    let mut columns: HashMap<Ident, LogicalColumn> = HashMap::new();

    for (col_name, col_type) in cols {
        match columns.get_key_value(&col_name) {
            Some((duplicate, _)) => errs.push_back(
                Diagnostic::spanned(
                    col_name.span(),
                    Level::Error,
                    format!(
                        "Cannot use name `{}` as it is already used. Names must be unique",
                        col_name
                    ),
                )
                .span_note(duplicate.span(), "Already used here".to_owned()),
            ),
            None => {
                columns.insert(
                    col_name,
                    LogicalColumn {
                        constraints: LogicalColumnConstraint {
                            read: true,
                            write: true,
                            unique: UniqueCons::NotUnique,
                        },
                        data_type: col_type,
                    },
                );
            }
        }
    }

    let mut constraint_names: HashSet<Ident> = HashSet::new();

    // Default constraints allow for insertion and deletion
    let mut constraints = LogicalRowConstraint {
        insert: true,
        delete: true,
        limit: None,
        genpk: None,
        preds: Vec::new(),
    };

    for Constraint {
        alias,
        method_span,
        expr,
    } in cons
    {
        if let Some(alias) = &alias {
            if let Some(duplicate) = constraint_names.get(alias) {
                errs.push_back(
                    Diagnostic::spanned(
                        alias.span(),
                        Level::Error,
                        format!(
                            "Cannot use name `{}` as it is already used. Names must be unique",
                            alias.to_string()
                        ),
                    )
                    .span_note(duplicate.span(), "Already used here".to_owned()),
                );
            } else {
                constraint_names.insert(alias.clone());
            }
        }

        // INV: if the alias is Some(..) then it is not a duplicate

        match expr {
            ConstraintExpr::Unique { field } => match columns.get_mut(&field) {
                Some(LogicalColumn {
                    constraints,
                    data_type,
                }) => match &constraints.unique {
                    UniqueCons::Unique(maybe_alias) => {
                        let mut diag = Diagnostic::spanned(
                            field.span(),
                            Level::Error,
                            format!("Column `{}` is already marked as unique", field),
                        );

                        if let Some(alias) = maybe_alias {
                            diag = diag.span_note(alias.span(), "Already marked here".to_owned());
                        }

                        errs.push_back(diag)
                    }
                    UniqueCons::NotUnique => {
                        constraints.unique = UniqueCons::Unique(alias); //sus
                    }
                },
                None => {
                    errs.push_back(
                        Diagnostic::spanned(
                            field.span(),
                            Level::Error,
                            format!("Column `{}` does not exist", field),
                        )
                        .span_note(method_span, "Unique constraint defined here".to_owned()),
                    );
                }
            },
            ConstraintExpr::Pred(expr) => {
                constraints.preds.push((expr, alias));
            }
            ConstraintExpr::GenPK { field } => {
                if let Some((ref column, ref other_alias)) = constraints.genpk {
                    let err: Diagnostic = Diagnostic::spanned(
                        field.span(),
                        Level::Error,
                        format!("Table {} already has a unique id", name.to_string()),
                    );
                    errs.push_back(
                        Diagnostic::spanned(
                            field.span(),
                            Level::Error,
                            format!("Table {} already has a unique id", name.to_string()),
                        )
                        .span_note(column.span(), "Already marked here".to_owned()),
                    );
                } else {
                    constraints.genpk = Some((field, alias));
                }
            }
            ConstraintExpr::Limit { size } => {
                if let Some((ref limit, ref other_alias)) = constraints.limit {
                    errs.push_back(
                        Diagnostic::spanned(
                            size.span(),
                            Level::Error,
                            "Limit constraint already defined".to_owned(),
                        )
                        .span_note(limit.span(), "Already defined here".to_owned()),
                    );
                } else {
                    constraints.limit = Some((size, alias));
                }
            }
        }
    }

    if errs.len() > 0 {
        Err(errs)
    } else {
        Ok(LogicalTable {
            name,
            constraints,
            columns,
        })
    }
}

fn translate_query(
    Query {
        name,
        params,
        streams,
    }: Query,
    lp: &mut LogicalPlan,
) -> Result<(), LinkedList<Diagnostic>> {
    // let query = lp.queries.insert(LogicalQuery {
    //     name,
    //     params: todo!(),
    //     returnval: todo!(),
    // })

    // TODO: implement

    todo!()
}
