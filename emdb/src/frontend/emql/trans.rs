use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet, LinkedList},
    mem,
};

use crate::{
    backend::{BackendTypes, GraphViz, Simple},
    plan::{
        helpers::*,
        repr::{
            Edge, EdgeKey, LogicalColumn, LogicalColumnConstraint, LogicalOp, LogicalOperator,
            LogicalPlan, LogicalQuery, LogicalQueryParams, LogicalRowConstraint, LogicalTable,
            OpKey, QueryKey, Record, RecordData, TableAccess, TableKey, UniqueCons,
        },
    },
    utils::misc::singlelist,
};
use proc_macro2::{Ident, Span};
use proc_macro_error::{Diagnostic, Level};
use syn::{spanned::Spanned, Expr};

use super::ast::{
    Ast, BackendImpl, BackendKind, Connector, Constraint, ConstraintExpr, FuncOp, Operator, Query,
    StreamExpr, Table,
};

fn ast_to_logical(
    Ast {
        backends,
        tables,
        queries,
    }: Ast,
) -> Result<LogicalPlan, LinkedList<Diagnostic>> {
    let mut errors = LinkedList::new();
    let mut lp = LogicalPlan::new();
    let mut qs = HashSet::new();
    let mut tn = HashMap::new();
    let mut bks = HashMap::new();

    for table in tables {
        errors.append(&mut add_table(&mut lp, &mut tn, table));
    }

    for query in queries {
        errors.append(&mut add_query(&mut lp, &mut qs, &tn, query));
    }

    for backend in backends {
        errors.append(&mut add_backend(&mut bks, backend));
    }

    if errors.is_empty() {
        Ok(lp)
    } else {
        Err(errors)
    }
}

fn add_backend(
    bks: &mut HashMap<Ident, BackendTypes>,
    BackendImpl { name, target }: BackendImpl,
) -> LinkedList<Diagnostic> {
    let mut errors = LinkedList::new();
    if let Some((other_name, _)) = bks.get_key_value(&name) {
        errors.push_back(
            Diagnostic::spanned(
                name.span(),
                Level::Error,
                format!("Redefinition of backend {}", name),
            )
            .span_error(
                other_name.span(),
                format!("{} originally defined here", name),
            ),
        );
    } else {
        bks.insert(
            name,
            match target {
                BackendKind::Graph => BackendTypes::GraphViz(GraphViz),
                BackendKind::Simple => BackendTypes::Simple(Simple),
            },
        );
    }
    errors
}

fn add_table(
    lp: &mut LogicalPlan,
    tn: &mut HashMap<Ident, TableKey>,
    Table { name, cols, cons }: Table,
) -> LinkedList<Diagnostic> {
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

    let tk = lp.tables.insert(LogicalTable {
        name: name.clone(),
        constraints,
        columns,
    });

    if let Some((other_t, _)) = tn.get_key_value(&name) {
        errs.push_back(
            Diagnostic::spanned(
                name.span(),
                Level::Error,
                format!("Redefinition of table {}", name),
            )
            .span_error(other_t.span(), format!("{} originally defined here", name)),
        );
    } else {
        tn.insert(name, tk);
    }

    errs
}

struct Return {
    span: Span,
    index: OpKey,
}

#[derive(Clone)]
struct Continue {
    data_type: Record,
    prev_edge: EdgeKey,
    last_span: Span,
}

enum StreamContext {
    Nothing { last_span: Span },
    Returned(Return),
    Continue(Continue),
}

enum VarState {
    Used { created: Span, used: Span },
    Available { created: Span, state: Continue },
}

/// Adds a new query from the AST to the logical plan
///
/// Checks for:
/// - Duplicate query names
/// - DUplicate returns
fn add_query(
    lp: &mut LogicalPlan,
    qs: &mut HashSet<Ident>,
    tn: &HashMap<Ident, TableKey>,
    Query {
        name,
        params,
        streams,
    }: Query,
) -> LinkedList<Diagnostic> {
    let mut errors = LinkedList::new();

    // if the name is duplicated, we can still analyse the query & add to the logical plan
    if let Some(other_q) = qs.get(&name) {
        errors.push_back(
            Diagnostic::spanned(
                name.span(),
                Level::Error,
                format!("Redefinition of query {}", name),
            )
            .span_error(other_q.span(), format!("{} originally defined here", name)),
        );
    } else {
        qs.insert(name.clone());
    }

    let params = params
        .into_iter()
        .map(|(name, data_type)| LogicalQueryParams { name, data_type })
        .collect();
    let qk = lp.queries.insert(LogicalQuery {
        name: name.clone(),
        params,
        returnval: None,
    });
    let mut vs = HashMap::new();
    let mut ret: Option<Return> = None;

    for stream in streams {
        match build_streamexpr(lp, tn, qk, &mut vs, stream) {
            Ok(r) => match (&mut ret, r) {
                (Some(prev_ret), Some(new_ret)) => {
                    errors.push_back(
                        Diagnostic::spanned(
                            new_ret.span,
                            Level::Error,
                            format!("multiple return statements for {}", name),
                        )
                        .span_error(prev_ret.span, format!("Previous return for {} here", name)),
                    );
                }
                (Some(_), None) => (),
                (None, rs) => ret = rs,
            },
            Err(mut es) => errors.append(&mut es),
        }
    }

    if let Some(ret) = ret {
        lp.queries[qk].returnval = Some(ret.index);
    }

    errors
}

/// helper for extracting a map of unique fields by Ident
fn extract_fields<T>(fields: Vec<(Ident, T)>) -> Result<HashMap<Ident, T>, LinkedList<Diagnostic>> {
    let mut map_fields: HashMap<Ident, T> = HashMap::with_capacity(fields.len());
    let mut errors = LinkedList::new();
    for (id, content) in fields {
        if let Some((other_id, _)) = map_fields.get_key_value(&id) {
            errors.push_back(
                Diagnostic::spanned(
                    id.span(),
                    Level::Error,
                    format!("Duplicate field present {}", id),
                )
                .span_error(other_id.span(), "Already defined here".to_owned()),
            );
        } else {
            map_fields.insert(id, content);
        }
    }

    if errors.is_empty() {
        Ok(map_fields)
    } else {
        Err(errors)
    }
}

fn build_streamexpr(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    StreamExpr { op, con }: StreamExpr,
) -> Result<Option<Return>, LinkedList<Diagnostic>> {
    match build_op_start(lp, tn, qk, vs, op) {
        Ok(ctx) => {
            let mut errors = LinkedList::new();
            match recur_stream(lp, tn, qk, vs, ctx, &mut errors, con) {
                Ok(res) => Ok(res),
                Err(()) => Err(errors),
            }
        }
        Err(es) => Err(es),
    }
}

fn recur_stream(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    ctx: StreamContext,
    errs: &mut LinkedList<Diagnostic>, // TODO: thunkify or convert to loop. We mutate a passed list to make tail-call elimination easier (but rust has no TCO yet)
    con: Option<(Connector, Box<StreamExpr>)>,
) -> Result<Option<Return>, ()> {
    match (ctx, con) {
        // No more operators (no data in, or data discarded)
        (StreamContext::Nothing { last_span }, None) => Ok(None),
        (StreamContext::Continue { .. }, None) => Ok(None),

        // Returning data from the query
        (StreamContext::Returned(ret), None) => Ok(Some(ret)),

        // Continue to next operator
        (
            StreamContext::Continue(Continue {
                data_type,
                prev_edge,
                last_span,
            }),
            Some((conn, nxt)),
        ) => {
            if data_type.stream != conn.stream {
                errs.push_back(
                    Diagnostic::spanned(
                        conn.span,
                        Level::Error,
                        if data_type.stream {
                            "Expected a stream, but found a single connector"
                        } else {
                            "Expected a single, but found a stream connector"
                        }
                        .to_owned(),
                    )
                    .span_note(last_span.clone(), "Previous stream type".to_owned()),
                );
            }

            let StreamExpr { op, con } = *nxt;

            match build_op_continue(
                lp,
                tn,
                qk,
                vs,
                Continue {
                    data_type,
                    prev_edge,
                    last_span,
                },
                op,
            ) {
                Ok(new_ctx) => recur_stream(lp, tn, qk, vs, new_ctx, errs, con),
                Err(mut es) => {
                    errs.append(&mut es);
                    Err(())
                }
            }
        }

        // Invalid Combos
        (StreamContext::Nothing { last_span }, Some((conn, _))) => {
            errs.push_back(
                Diagnostic::spanned(
                    last_span,
                    Level::Error,
                    "No output data provided for next operator".to_owned(),
                )
                .span_error(
                    conn.span,
                    format!(
                        "Expected a {} out here",
                        if conn.stream {
                            "stream of data"
                        } else {
                            "singe data"
                        }
                    ),
                ),
            );
            Err(())
        }
        (StreamContext::Returned(r), Some((conn, _))) => {
            errs.push_back(
                Diagnostic::spanned(
                    r.span,
                    Level::Error,
                    format!("Early return leaves no data for the next operator"),
                )
                .span_error(
                    conn.span,
                    format!(
                        "Expected a {} out here",
                        if conn.stream {
                            "stream of data"
                        } else {
                            "singe data"
                        }
                    ),
                ),
            );
            Err(())
        }
    }
}

fn build_op_start(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    op: Operator,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    fn error_cannot_start(name: &str, span: Span) -> LinkedList<Diagnostic> {
        singlelist(Diagnostic::spanned(
            span,
            Level::Error,
            format!(
                "Cannot start stream from a {}, only use, ref and unique(..) can start streams",
                name
            ),
        ))
    }
    match op {
        // start the stream
        Operator::Ref {
            ref_span,
            table_name,
        } => {
            // Needs to come from a valid table
            if let Some(table_id) = tn.get(&table_name) {
                let data_type = Record {
                    fields: HashMap::from([(table_name, RecordData::Ref(*table_id))]),
                    stream: true,
                };

                let out_edge = lp.operator_edges.insert(Edge::Null);

                let ref_op = lp.operators.insert(LogicalOperator {
                    query: Some(qk),
                    operator: LogicalOp::Scan {
                        access: TableAccess::Ref,
                        table: *table_id,
                        output: out_edge,
                    },
                });

                lp.operator_edges[out_edge] = Edge::Uni {
                    from: ref_op,
                    with: data_type.clone(),
                };

                Ok(StreamContext::Continue(Continue {
                    data_type,
                    prev_edge: out_edge,
                    last_span: ref_span,
                }))
            } else {
                Err(singlelist(Diagnostic::spanned(
                    ref_span,
                    Level::Error,
                    format!("No such table {}", table_name),
                )))
            }
        }
        Operator::Use { use_span, var_name } => {
            if let Some(table_id) = tn.get(&var_name) {
                let data_type = lp.tables.get(*table_id).unwrap().get_all_cols_type();
                let out_edge = lp.operator_edges.insert(Edge::Null);
                let use_op = lp.operators.insert(LogicalOperator {
                    query: Some(qk),
                    operator: LogicalOp::Scan {
                        access: TableAccess::AllCols,
                        table: *table_id,
                        output: out_edge,
                    },
                });
                lp.operator_edges[out_edge] = Edge::Uni {
                    from: use_op,
                    with: data_type.clone(),
                };

                Ok(StreamContext::Continue(Continue {
                    data_type,
                    prev_edge: out_edge,
                    last_span: use_span,
                }))
            } else if let Some(var) = vs.get_mut(&var_name) {
                match var {
                    VarState::Used { created, used } => Err(singlelist(
                        Diagnostic::spanned(
                            use_span,
                            Level::Error,
                            format!("Variable {} has already been used", var_name),
                        )
                        .span_error(*created, "Was created here".to_owned())
                        .span_error(*used, "And consumed here".to_owned()),
                    )),
                    VarState::Available { created, state } => {
                        let ret = Ok(StreamContext::Continue(state.clone()));
                        *var = VarState::Used {
                            created: *created,
                            used: use_span,
                        };
                        ret
                    }
                }
            } else {
                Err(singlelist(Diagnostic::spanned(
                    use_span,
                    Level::Error,
                    format!(
                        "There are no variables or tables called {} available to use",
                        var_name
                    ),
                )))
            }
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
            if let Some(table_id) = tn.get(&table) {
                let out_edge = lp.operator_edges.insert(Edge::Null);

                let (unique_op, data_type) = if refs {
                    (
                        lp.operators.insert(LogicalOperator {
                            query: Some(qk),
                            operator: LogicalOp::Unique {
                                unique_field,
                                access: TableAccess::Ref,
                                from_expr,
                                table: *table_id,
                                output: out_edge,
                            },
                        }),
                        Record {
                            fields: HashMap::from([(table, RecordData::Ref(*table_id))]),
                            stream: true,
                        },
                    )
                } else {
                    (
                        lp.operators.insert(LogicalOperator {
                            query: Some(qk),
                            operator: LogicalOp::Unique {
                                unique_field,
                                access: TableAccess::AllCols,
                                from_expr,
                                table: *table_id,
                                output: out_edge,
                            },
                        }),
                        lp.tables.get(*table_id).unwrap().get_all_cols_type(),
                    )
                };

                lp.operator_edges[out_edge] = Edge::Uni {
                    from: unique_op,
                    with: data_type.clone(),
                };

                Ok(StreamContext::Continue(Continue {
                    data_type,
                    prev_edge: out_edge,
                    last_span: fn_span,
                }))
            } else {
                Err(singlelist(Diagnostic::spanned(
                    table.span(),
                    Level::Error,
                    format!("No table named {} found", table),
                )))
            }
        }

        // otherwise
        Operator::FuncOp { fn_span, op } => Err(error_cannot_start("function", fn_span)),
        Operator::Ret { ret_span } => Err(error_cannot_start("return", ret_span)),
        Operator::Let { let_span, var_name } => Err(error_cannot_start("let", let_span)),
    }
}

fn build_op_continue(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    prev: Continue,
    curr_op: Operator,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    fn cannot_use_op(name: &str, span: Span) -> LinkedList<Diagnostic> {
        singlelist(Diagnostic::spanned(
            span,
            Level::Error,
            format!("Cannot use {} in the middle of a stream", name),
        ))
    }

    match curr_op {
        // starting a stream
        Operator::Ref {
            ref_span,
            table_name,
        } => Err(cannot_use_op("ref", ref_span)),
        Operator::Use { use_span, var_name } => Err(cannot_use_op("use", use_span)),
        Operator::FuncOp {
            fn_span,
            op:
                FuncOp::Unique {
                    table,
                    refs,
                    unique_field,
                    from_expr,
                },
        } => Err(cannot_use_op("unique", fn_span)),

        // operations on the stream
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Update { reference, fields },
        } => build_update(lp, tn, qk, vs, prev, reference, fields),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Insert { table_name },
        } => todo!(),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Delete,
        } => todo!(),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Map { new_fields },
        } => todo!(),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Filter(_),
        } => todo!(),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Row { fields },
        } => todo!(),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Sort { fields },
        } => todo!(),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Fold { initial, update },
        } => todo!(),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Assert(_),
        } => todo!(),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Collect,
        } => todo!(),

        // ending the stream
        Operator::Ret { ret_span } => {
            let return_op = lp.operators.insert(LogicalOperator {
                query: Some(qk),
                operator: LogicalOp::Return {
                    input: prev.prev_edge,
                },
            });

            Ok(StreamContext::Returned(Return {
                span: ret_span,
                index: return_op,
            }))
        }

        Operator::Let { let_span, var_name } => {
            if let Some(varstate) = vs.get(&var_name) {
                match varstate {
                    VarState::Used { created, used } => Err(singlelist(
                        Diagnostic::spanned(
                            let_span,
                            Level::Error,
                            format!("Cannot assign to already created variable {}", var_name),
                        )
                        .span_error(*created, "Created here".to_owned())
                        .span_error(*used, "used here".to_owned()),
                    )),
                    VarState::Available { created, state } => Err(singlelist(
                        Diagnostic::spanned(
                            let_span,
                            Level::Error,
                            format!("Cannot assign to already created variable {}", var_name),
                        )
                        .span_error(*created, "Created here".to_owned()),
                    )),
                }
            } else {
                vs.insert(
                    var_name,
                    VarState::Available {
                        created: let_span,
                        state: prev,
                    },
                );
                Ok(StreamContext::Nothing {
                    last_span: let_span,
                })
            }
        }
    }
}

fn build_update(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    prev: Continue,

    // include the fields that are part of the AST
    reference: Ident,
    fields: Vec<(Ident, Expr)>,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    todo!()
}

// TODO: work out better structure for the build_update, build_assert etc functions
// TODO: work out synthetic types representation
