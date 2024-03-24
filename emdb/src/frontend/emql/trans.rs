use std::collections::{HashMap, HashSet, LinkedList};

use proc_macro2::{Ident, Span};
use proc_macro_error::{Diagnostic, Level};
use syn::{spanned::Spanned, Expr, Type};

use crate::{
    frontend::emql::ast::{Ast, ConstraintExpr, FuncOp, StreamExpr},
    plan::{
        repr::{
            GenIndex, LogicalColumn, LogicalColumnConstraint, LogicalOperator, LogicalPlan, LogicalQuery, LogicalRowConstraint, LogicalTable, Record, RecordData, TableAccess, UniqueCons
        },
        targets::{Target, Targets},
    },
    utils::misc::singlelist,
};

use super::ast::{BackendImpl, Constraint, Operator, Query, Table};

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

    for query in queries {
        match translate_query(query, &mut logical_plan) {
            Ok(_) => (), // TODO COMPLETE
            Err(mut query_errs) => {
                errs.append(&mut query_errs);
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
pub(super) fn add_table(
    lp: &mut LogicalPlan,
    Table { name, cols, cons }: Table,
) -> Option<LinkedList<Diagnostic>> {
    let mut errs = LinkedList::new();
    
    // Add each column, checking for duplicate names
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
                            unique: UniqueCons::NotUnique,
                        },
                        data_type: col_type,
                    },
                );
            }
        }
    }

    // Add each constraint, checking aliases used are unique.
    let mut constraint_names: HashSet<Ident> = HashSet::new();
    let mut constraints = LogicalRowConstraint {
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

    lp.tables.insert(LogicalTable {
        name,
        constraints,
        columns,
    });

    if errs.len() > 0 {
        Some(errs)
    } else {
        None
    }
}

enum AvailName {
    Table(GenIndex<LogicalTable>),
    Variable {
        used: Option<Ident>,
        data_type: Record,
        location: GenIndex<LogicalOperator>,
    },
}

fn add_query(
    lp: &mut LogicalPlan,
    Query {
        name,
        params,
        streams,
    }: Query,
) -> Result<(), LinkedList<Diagnostic>> {
    // A map of all used names, starts with the tables, includes variables
    // INV: never remove a name from this map
    let mut avail_names: HashMap<Ident, AvailName> = lp
        .tables
        .iter()
        .map(|(id, LogicalTable { name, .. })| (name.clone(), AvailName::Table(id)))
        .collect::<HashMap<_, _>>();

    let mut errs = LinkedList::new();

    // INV: only one return statement can be present
    let mut return_used = false;

    for StreamExpr { op, con } in streams {
        match translate_streamexpr_first(&mut avail_names, lp, op) {
            Err(mut es) => errs.append(&mut es),
            Ok((op_index, data_type)) => {

            }
        }
    }

    if errs.len() > 0 {
        Err(errs)
    } else {
        Ok(())
    }
}

fn tran_add_query(
    lp: &mut LogicalPlan,
    Query {
        name,
        params,
        streams,
    }: Query,
) -> Option<LinkedList<Diagnostic>> {
    todo!()
}

/// Translate the first operator in a stream expression
/// - Includes only the access and row creation operators (all of which have a single output)
fn translate_streamexpr_first(
    lp: &mut LogicalPlan,
    avail_names: &mut HashMap<Ident, AvailName>,
    op: Operator,
) -> Result<(GenIndex<LogicalOperator>, Record), LinkedList<Diagnostic>> {
    match op {
        Operator::Use { use_span, var_name } => match avail_names.get_mut(&var_name) {
            Some(name) => match name {
                AvailName::Table(t) => Ok((
                    lp.operators.insert(LogicalOperator::Scan {
                        refs: TableAccess::AllCols,
                        table: t.clone(),
                        output: None,
                    }),
                    lp.tables.get(*t).expect("INV: table must exist").get_type(),
                )),
                AvailName::Variable {
                    data_type,
                    used,
                    location,
                } => {
                    if used.is_some() {
                        Err(singlelist(Diagnostic::spanned(
                            var_name.span(),
                            Level::Error,
                            format!("Variable `{}` already used", var_name),
                        )))
                    } else {
                        *used = Some(var_name.clone());
                        Ok((*location, data_type.clone()))
                    }
                }
            },
            None => Err(singlelist(Diagnostic::spanned(
                var_name.span(),
                Level::Error,
                format!("Unknown table or variable {} used", var_name),
            ))),
        },
        Operator::Ref {
            ref_span,
            table_name,
        } => match avail_names.get(&table_name) {
            Some(name) => match name {
                AvailName::Table(t) => Ok((
                    lp.operators.insert(LogicalOperator::Scan {
                        refs: true,
                        table: t.clone(),
                        output: None,
                    }),
                    lp.tables.get(*t).expect("INV: table must exist").get_type(),
                )),
                AvailName::Variable {
                    data_type,
                    used,
                    location,
                } => Err({
                    let d = Diagnostic::spanned(
                        ref_span,
                        Level::Error,
                        format!("Cannot stream references from a variable {}", table_name),
                    );

                    singlelist(if let Some(before) = used {
                        d.span_note(
                            before.span(),
                            "Furthermore variable was already used here".to_owned(),
                        )
                    } else {
                        d
                    })
                }),
            },

            None => Err(singlelist(Diagnostic::spanned(
                table_name.span(),
                Level::Error,
                format!("Unknown table {} referenced", table_name),
            ))),
        },
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Row { fields },
        } => {
            let (new_fields, record) = deduplicate_row_fields(fields)?;
            Ok((
                lp.operators.insert(LogicalOperator::Row {
                    fields: new_fields,
                    output: None,
                }),
                record,
            ))
        }
        Operator::FuncOp {
            fn_span,
            op:
                FuncOp::Unique {
                    table,
                    refs,
                    unique_field,
                    from_expr,
                },
        } => {
            // Need to check that the unique constraint is applied to the relevant table.
            match avail_names.get(&table) {
                Some(AvailName::Table(t)) => {
                    let table = lp.tables.get(*t).expect("INV: table must exist");
                    if let Some(LogicalColumn { constraints, .. }) =
                        table.columns.get(&unique_field)
                    {
                        if let UniqueCons::Unique(_) = constraints.unique {
                            Ok((
                                lp.operators.insert(LogicalOperator::Unique {
                                    table: t.clone(),
                                    access: if refs {TableAccess::Ref} else {TableAccess::AllCols},
                                    unique_field,
                                    from_expr,
                                    output: None,
                                }),
                                table.get_type(),
                            ))
                        } else {
                            Err(singlelist(Diagnostic::spanned(
                                unique_field.span(),
                                Level::Error,
                                format!("Field `{}` is not unique", unique_field),
                            )))
                        }
                    } else {
                        Err(singlelist(Diagnostic::spanned(
                            unique_field.span(),
                            Level::Error,
                            format!("Field `{}` does not exist", unique_field),
                        )))
                    }
                }
                Some(AvailName::Variable { used, .. }) => Err(singlelist(Diagnostic::spanned(
                    table.span(),
                    Level::Error,
                    format!("Cannot use variable `{}` as a table", table),
                ))),
                None => Err(singlelist(Diagnostic::spanned(
                    table.span(),
                    Level::Error,
                    format!("Unknown table `{}`", table),
                ))),
            }
        }
        other => {
            let (span, name) = extract_span_name(other);
            Err(singlelist(Diagnostic::spanned(
                span,
                Level::Error,
                format!("Cannot use {} at the start of a stream expression", name),
            )))
        }
    }
}

fn deduplicate_row_fields(
    fields: Vec<(Ident, Type, Expr)>,
) -> Result<(HashMap<Ident, (Type, Expr)>, Record), LinkedList<Diagnostic>> {
    let mut new_fields = HashMap::new();
    let mut errs = LinkedList::new();

    for (name, data_type, expr) in fields {
        if let Some(dup) = new_fields.insert(name.clone(), (data_type, expr)) {
            errs.push_back(Diagnostic::spanned(
                name.span(),
                Level::Error,
                format!("Field `{}` is already defined", name),
            ));
        }
    }

    if errs.len() > 0 {
        Err(errs)
    } else {
        let record = Record {
            fields: new_fields
                .iter()
                .map(|(a, (t, _))| (a.clone(), RecordData::Rust(t.clone())))
                .collect(),
            stream: false,
        };
        Ok((new_fields, record))
    }
}

fn extract_span_name(op: Operator) -> (Span, &'static str) {
    match op {
        Operator::Ret { ret_span } => (ret_span, "return"),
        Operator::Ref {
            ref_span,
            table_name,
        } => (ref_span, "table reference"),
        Operator::Let { let_span, var_name } => (let_span, "variable declaration"),
        Operator::Use { use_span, var_name } => (use_span, "variable or table usage"),
        Operator::FuncOp { fn_span, op } => (
            fn_span,
            match op {
                FuncOp::Update { reference, fields } => "update",
                FuncOp::Insert { table_name } => "insert",
                FuncOp::Delete => "delete",
                FuncOp::Map { new_fields } => "map",
                FuncOp::Unique {
                    table,
                    refs,
                    unique_field,
                    from_expr,
                } => "unique",
                FuncOp::Filter(_) => "filter",
                FuncOp::Row { fields } => "row",
                FuncOp::Sort { fields } => "sort",
                FuncOp::Fold { initial, update } => "fold",
                FuncOp::Assert(_) => "assertion",
                FuncOp::Collect => "collect",
            },
        ),
    }
}


#[cfg(test)]
mod test {
    use super::*;
}