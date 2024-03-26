use crate::{
    backend::{BackendTypes, GraphViz, Simple},
    frontend::emql::{
        ast::{
            Ast, BackendImpl, BackendKind, Connector, Constraint, ConstraintExpr, FuncOp, Operator,
            Query, StreamExpr, Table,
        },
        errors,
    },
    plan::repr::{
        Edge, EdgeKey, LogicalColumn, LogicalColumnConstraint, LogicalOp, LogicalOperator,
        LogicalPlan, LogicalQuery, LogicalQueryParams, LogicalRowConstraint, LogicalTable, OpKey,
        QueryKey, Record, RecordData, TableAccess, TableKey, UniqueCons,
    },
    utils::misc::singlelist,
};
use proc_macro2::{Ident, Span};
use proc_macro_error::{Diagnostic, Level};
use std::collections::{HashMap, HashSet, LinkedList};
use syn::Expr;

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
        errors.push_back(errors::backend_redefined(&name, other_name));
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
            Some((duplicate, _)) => {
                errs.push_back(errors::table_column_redefined(&col_name, duplicate))
            }
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
                errs.push_back(errors::table_constraint_alias_redefined(alias, duplicate));
            } else {
                constraint_names.insert(alias.clone());
            }
        }
        match expr {
            ConstraintExpr::Unique { field } => match columns.get_mut(&field) {
                Some(LogicalColumn {
                    constraints,
                    data_type,
                }) => match &constraints.unique {
                    UniqueCons::Unique(maybe_alias) => {
                        errs.push_back(errors::table_constraint_duplicate_unique(
                            &field,
                            method_span,
                            maybe_alias,
                        ));
                    }
                    UniqueCons::NotUnique => {
                        constraints.unique = UniqueCons::Unique(alias);
                    }
                },
                None => errs.push_back(errors::table_constraint_nonexistent_unique_column(
                    &alias,
                    &field,
                    &name,
                    method_span,
                )),
            },
            ConstraintExpr::Pred(expr) => {
                constraints.preds.push((expr, alias));
            }
            ConstraintExpr::Limit { size } => {
                if let Some((ref limit, ref other_alias)) = constraints.limit {
                    errs.push_back(errors::table_constraint_duplicate_limit(
                        &alias,
                        &name,
                        method_span,
                    ));
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
        errs.push_back(errors::table_redefined(&name, other_t));
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
        errors.push_back(errors::query_redefined(&name, other_q));
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
                    errors.push_back(errors::query_multiple_returns(
                        new_ret.span,
                        prev_ret.span,
                        &name,
                    ));
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
            errors.push_back(errors::query_operator_field_redefined(&id, other_id));
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
                errs.push_back(errors::query_stream_single_connection(
                    conn.span,
                    last_span,
                    data_type.stream,
                ));
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
            errs.push_back(errors::query_no_data_for_next_operator(
                conn.span,
                conn.stream,
                last_span,
            ));
            Err(())
        }
        (StreamContext::Returned(r), Some((conn, _))) => {
            errs.push_back(errors::query_early_return(conn.span, conn.stream, r.span));
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

                let logical_table = lp.tables.get(*table_id).unwrap();

                if let Some(using_col) = logical_table.columns.get(&unique_field) {
                    if let UniqueCons::Unique(_) = using_col.constraints.unique {
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
                                    stream: false,
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
                                logical_table.get_all_cols_type(),
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
                            unique_field.span(),
                            Level::Error,
                            format!("Field {} is not unique", unique_field),
                        )))
                    }
                } else {
                    Err(singlelist(Diagnostic::spanned(
                        unique_field.span(),
                        Level::Error,
                        format!(
                            "No field named {} found in table {}",
                            unique_field, logical_table.name
                        ),
                    )))
                }
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
        } => build_insert(lp, tn, qk, vs, prev, table_name),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::DeRef { reference, named },
        } => build_deref(lp, tn, qk, vs, prev, fn_span, reference, named),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Delete,
        } => build_delete(lp, tn, qk, vs, prev),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Map { new_fields },
        } => build_map(lp, tn, qk, vs, prev),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Filter(_),
        } => build_filter(lp, tn, qk, vs, prev),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Row { fields },
        } => build_row(lp, tn, qk, vs, prev),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Sort { fields },
        } => build_sort(lp, tn, qk, vs, prev),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Fold { initial, update },
        } => build_fold(lp, tn, qk, vs, prev),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Assert(_),
        } => build_assert(lp, tn, qk, vs, prev),
        Operator::FuncOp {
            fn_span,
            op: FuncOp::Collect,
        } => build_collect(lp, tn, qk, vs, prev),

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
    let mut errors = LinkedList::new();

    // check fields are unique
    let raw_fields = match extract_fields(fields) {
        Ok(f) => Some(f),
        Err(mut es) => {
            errors.append(&mut es);
            None
        }
    };

    // get the table to update
    let raw_table_id = match prev.data_type.fields.get(&reference) {
        Some(RecordData::Ref(table)) => Some(table),
        Some(_) => {
            errors.push_back(Diagnostic::spanned(
                reference.span(),
                Level::Error,
                format!(
                    "Can only updating using a table reference, but {} doe not reference a table",
                    reference
                ),
            ));
            None
        }
        None => {
            errors.push_back(Diagnostic::spanned(
                reference.span(),
                Level::Error,
                format!("No available field {}", reference),
            ));
            None
        }
    };

    if let (Some(table_id), Some(nondup_fields)) = (raw_table_id, raw_fields) {
        // TODO: we could check the non-duplicate fields, cost/benefit unclear

        // check all fields are available
        let table = lp.tables.get(*table_id).unwrap();

        for (id, _) in nondup_fields.iter() {
            if !table.columns.contains_key(id) {
                errors.push_back(
                    Diagnostic::spanned(
                        id.span(),
                        Level::Error,
                        format!(
                            "No such field {} in table {} (from {})",
                            id, table.name, reference
                        ),
                    )
                    .span_note(table.name.span(), "Table defined here".to_owned()),
                );
            }
        }

        if errors.is_empty() {
            let out_edge = lp.operator_edges.insert(Edge::Null);
            let update_op = lp.operators.insert(LogicalOperator {
                query: Some(qk),
                operator: LogicalOp::Update {
                    input: prev.prev_edge,
                    reference: reference.clone(),
                    table: *table_id,
                    mapping: nondup_fields,
                    output: out_edge,
                },
            });

            let out_data_type = Record {
                fields: HashMap::from([(reference.clone(), RecordData::Ref(*table_id))]),
                stream: prev.data_type.stream,
            };

            lp.operator_edges[out_edge] = Edge::Uni {
                from: update_op,
                with: out_data_type.clone(),
            };

            Ok(StreamContext::Continue(Continue {
                data_type: out_data_type,
                prev_edge: out_edge,
                last_span: reference.span(),
            }))
        } else {
            Err(errors)
        }
    } else {
        Err(errors)
    }
}

// TODO: work out better structure for the build_update, build_assert etc functions
// TODO: work out synthetic types representation

fn build_insert(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    Continue {
        data_type: Record { mut fields, stream },
        prev_edge,
        last_span,
    }: Continue,

    // include the fields that are part of the AST
    table_name: Ident,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    if let Some(table_id) = tn.get(&table_name) {
        let table = lp.tables.get(*table_id).unwrap();
        let mut errors = LinkedList::new();

        for (id, col) in table.columns.iter() {
            match fields.remove(id) {
                Some(RecordData::Rust(r)) => {
                    if r != col.data_type {
                        errors.push_back(
                            Diagnostic::spanned(
                                id.span(),
                                Level::Error,
                                format!("Type mismatch for field {} in table {}", id, table_name),
                            )
                            .span_note(
                                id.span(),
                                format!("Field {} is defined as {:?}", id, col.data_type),
                            ),
                        );
                    }
                }
                Some(RecordData::Ref(_) | RecordData::Record(_)) => {
                    errors.push_back(
                        Diagnostic::spanned(
                            id.span(),
                            Level::Error,
                            format!(
                                "Field {} is not the correct type {:?}, or a rust type",
                                id, col.data_type
                            ),
                        )
                        .span_note(
                            id.span(),
                            format!("Field {} is defined as {:?}", id, col.data_type),
                        ),
                    );
                }
                None => {
                    errors.push_back(
                        Diagnostic::spanned(
                            table_name.span(),
                            Level::Error,
                            format!("Field {} is missing from the insert", id),
                        )
                        .span_note(id.span(), format!("{} is defined here", id))
                        .span_note(
                            last_span,
                            format!("{} should have been part of the output from here", id),
                        ),
                    );
                }
            }
        }

        for (id, _) in fields.iter() {
            errors.push_back(
                Diagnostic::spanned(
                    id.span(),
                    Level::Error,
                    format!("Field {} is not a valid field for table {}", id, table_name),
                )
                .span_note(
                    id.span(),
                    format!("Field {} is not a valid field for table {}", id, table_name),
                ),
            );
        }

        if errors.is_empty() {
            let out_edge = lp.operator_edges.insert(Edge::Null);
            let insert_op = lp.operators.insert(LogicalOperator {
                query: Some(qk),
                operator: LogicalOp::Insert {
                    input: prev_edge,
                    table: *table_id,
                    output: out_edge,
                },
            });

            let out_data_type = Record {
                fields: HashMap::from([(table_name.clone(), RecordData::Ref(*table_id))]),
                stream,
            };

            lp.operator_edges[out_edge] = Edge::Uni {
                from: insert_op,
                with: out_data_type.clone(),
            };

            Ok(StreamContext::Continue(Continue {
                data_type: out_data_type,
                prev_edge: out_edge,
                last_span: table_name.span(),
            }))
        } else {
            Err(errors)
        }
    } else {
        Err(singlelist(Diagnostic::spanned(
            table_name.span(),
            Level::Error,
            format!("No table named {} found", table_name),
        )))
    }
}

#[allow(clippy::too_many_arguments)]
fn build_deref(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,

    // new last_span
    fn_span: Span,

    // include the fields that are part of the AST
    reference: Ident,
    named: Ident,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    if let Some(field) = data_type.fields.get(&named) {
        Err(singlelist(Diagnostic::spanned(
            named.span(),
            Level::Error,
            format!("Field {} already exists", named),
        )))
    } else if let Some(field_type) = data_type.fields.get(&reference) {
        match field_type {
            RecordData::Record(_) => Err(singlelist(Diagnostic::spanned(
                reference.span(),
                Level::Error,
                "Cannot dereference a record".to_owned(),
            ))),
            RecordData::Rust(_) => Err(singlelist(Diagnostic::spanned(
                reference.span(),
                Level::Error,
                "Cannot dereference a rust type".to_owned(),
            ))),
            RecordData::Ref(table_id) => {
                let table_id_copy = *table_id;
                let table_type = lp.tables.get(*table_id).unwrap().get_all_cols_type();

                let Record { mut fields, stream } = data_type;
                fields.insert(
                    named.clone(),
                    RecordData::Record(Record {
                        fields: table_type.fields,
                        stream: false,
                    }),
                );

                let new_type = Record { fields, stream };

                let out_edge = lp.operator_edges.insert(Edge::Null);
                let deref_op = lp.operators.insert(LogicalOperator {
                    query: Some(qk),
                    operator: LogicalOp::DeRef {
                        input: prev_edge,
                        reference,
                        named,
                        table: table_id_copy,
                        output: out_edge,
                    },
                });

                lp.operator_edges[out_edge] = Edge::Uni {
                    from: deref_op,
                    with: new_type.clone(),
                };

                Ok(StreamContext::Continue(Continue {
                    data_type: new_type,
                    prev_edge: out_edge,
                    last_span: fn_span,
                }))
            }
        }
    } else {
        Err(singlelist(Diagnostic::spanned(
            reference.span(),
            Level::Error,
            format!("No field {} found", reference),
        )))
    }
}

fn build_delete(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    todo!()
}

fn build_map(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    todo!()
}
fn build_filter(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    todo!()
}
fn build_row(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    todo!()
}
fn build_sort(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    todo!()
}
fn build_fold(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    todo!()
}
fn build_assert(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    todo!()
}
fn build_collect(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    todo!()
}
