use super::operators::build_logical;
use crate::{
    backend,
    frontend::emql::{
        ast::{
            Ast, AstType, BackendImpl, Connector, Constraint, ConstraintExpr, Query, StreamExpr,
            Table,
        },
        errors,
    },
    plan::{self, ColSelect, DataFlow},
};
use proc_macro2::{Ident, Span};
use proc_macro_error::Diagnostic;
use std::collections::{HashMap, HashSet, LinkedList};

pub(super) struct ReturnVal {
    pub span: Span,
    pub index: plan::Key<plan::Operator>,
}

#[derive(Clone)]
pub(super) struct Continue {
    pub data_type: plan::Data,
    pub prev_edge: plan::Key<plan::DataFlow>,
    pub last_span: Span,
}

pub(super) enum StreamContext {
    Nothing { last_span: Span },
    Returned(ReturnVal),
    Continue(Continue),
}

pub(super) enum VarState {
    Used { created: Span, used: Span },
    Available { created: Span, state: Continue },
}

pub fn ast_to_logical(
    Ast {
        backends,
        tables,
        queries,
    }: Ast,
) -> Result<(plan::Plan, backend::Targets), LinkedList<Diagnostic>> {
    let mut errors = LinkedList::new();
    let mut lp = plan::Plan::new();
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
        Ok((lp, backend::Targets { impls: bks }))
    } else {
        Err(errors)
    }
}

fn add_backend(
    bks: &mut HashMap<Ident, backend::Backend>,
    BackendImpl {
        impl_name,
        backend_name,
        options,
    }: BackendImpl,
) -> LinkedList<Diagnostic> {
    let (i, mut errors) = match backend::parse_options(backend_name, options) {
        Ok(b) => (Some(b), LinkedList::new()),
        Err(es) => (None, es),
    };

    if let Some((other_name, _)) = bks.get_key_value(&impl_name) {
        errors.push_back(errors::backend_redefined(&impl_name, other_name));
    } else if let Some(b) = i {
        bks.insert(impl_name, b);
    }
    errors
}

fn add_table(
    lp: &mut plan::Plan,
    tn: &mut HashMap<Ident, plan::Key<plan::Table>>,
    Table { name, cols, cons }: Table,
) -> LinkedList<Diagnostic> {
    let mut errs = LinkedList::new();

    // Add each column, checking for duplicate names
    let mut columns: HashMap<Ident, plan::Column> = HashMap::new();
    for (col_name, col_type) in cols {
        if let Some((duplicate, _)) = columns.get_key_value(&col_name) {
            errs.push_back(errors::table_column_redefined(&col_name, duplicate));
        } else {
            let type_index = lp
                .scalar_types
                .insert(plan::ConcRef::Conc(plan::ScalarTypeConc::Rust(col_type)));
            columns.insert(
                col_name,
                plan::Column {
                    cons: plan::ColumnConstraints { unique: None },
                    data_type: type_index,
                },
            );
        }
    }

    // Add each constraint, checking aliases used are unique.
    let mut constraint_names: HashSet<Ident> = HashSet::new();
    let mut row_cons = plan::RowConstraints {
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
                Some(plan::Column { cons, data_type }) => match &cons.unique {
                    Some(cons) => {
                        errs.push_back(errors::table_constraint_duplicate_unique(
                            &field,
                            method_span,
                            &cons.alias,
                        ));
                    }
                    None => {
                        cons.unique = Some(plan::Constraint {
                            alias,
                            cons: plan::Unique,
                        });
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
                row_cons.preds.push(plan::Constraint {
                    alias,
                    cons: plan::Pred(expr),
                });
            }
            ConstraintExpr::Limit { size } => {
                if let Some(plan::Constraint { alias, cons }) = &row_cons.limit {
                    errs.push_back(errors::table_constraint_duplicate_limit(
                        alias,
                        &name,
                        method_span,
                    ));
                } else {
                    row_cons.limit = Some(plan::Constraint {
                        alias,
                        cons: plan::Limit(size),
                    });
                }
            }
        }
    }

    let tk = lp.tables.insert(plan::Table {
        name: name.clone(),
        row_cons,
        columns,
    });

    if let Some((other_t, _)) = tn.get_key_value(&name) {
        errs.push_back(errors::table_redefined(&name, other_t));
    } else {
        tn.insert(name, tk);
    }

    errs
}

fn add_query(
    lp: &mut plan::Plan,
    qs: &mut HashSet<Ident>,
    tn: &HashMap<Ident, plan::Key<plan::Table>>,
    Query {
        name,
        params,
        streams,
    }: Query,
) -> LinkedList<Diagnostic> {
    let mut vs = HashMap::new();
    let mut ts = HashMap::new();
    let mut mo = None;
    let mut ret: Option<ReturnVal> = None;
    let (raw_params, mut errors) = extract_fields(params, errors::query_parameter_redefined);

    // if the name is duplicated, we can still analyse the query & add to the logical plan
    if let Some(other_q) = qs.get(&name) {
        errors.push_back(errors::query_redefined(&name, other_q));
    } else {
        qs.insert(name.clone());
    }

    let params = raw_params
        .into_iter()
        .filter_map(|(name, data_type)| {
            match ast_typeto_scalar(
                tn,
                &mut ts,
                data_type,
                |e| errors::query_param_ref_table_not_found(&name, e),
                errors::query_no_cust_type_found,
            ) {
                Ok(t) => Some((name, lp.scalar_types.insert(t))),
                Err(e) => {
                    errors.push_back(e);
                    None
                }
            }
        })
        .collect();

    let qk = lp.queries.insert(plan::Query {
        name: name.clone(),
        params,
        returnval: None,
    });

    for stream in streams {
        match build_streamexpr(lp, tn, qk, &mut vs, &mut ts, &mut mo, stream) {
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

    for (name, vs) in vs {
        if let VarState::Available { created, state } = vs {
            discard_continue(lp, qk, state);
        }
    }

    errors
}

fn build_streamexpr(
    lp: &mut plan::Plan,
    tn: &HashMap<Ident, plan::Key<plan::Table>>,
    qk: plan::Key<plan::Query>,
    vs: &mut HashMap<Ident, VarState>,
    ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
    mo: &mut Option<plan::Key<plan::Operator>>,
    StreamExpr { op, con }: StreamExpr,
) -> Result<Option<ReturnVal>, LinkedList<Diagnostic>> {
    match build_logical(op, lp, tn, qk, vs, ts, mo, None) {
        Ok(ctx) => {
            let mut errors = LinkedList::new();
            match recur_stream(lp, tn, qk, vs, ts, mo, ctx, &mut errors, con) {
                Ok(res) if errors.is_empty() => Ok(res),
                _ => Err(errors),
            }
        }
        Err(es) => Err(es),
    }
}

fn discard_continue(
    lp: &mut plan::Plan,
    qk: plan::Key<plan::Query>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
) {
    let discard_op = lp.operators.insert(plan::Operator {
        query: qk,
        kind: plan::OperatorKind::Flow(plan::FlowOperator::Discard { input: prev_edge }),
    });
    update_incomplete(lp.get_mut_dataflow(prev_edge), discard_op);
}

#[allow(clippy::too_many_arguments)]
fn recur_stream(
    lp: &mut plan::Plan,
    tn: &HashMap<Ident, plan::Key<plan::Table>>,
    qk: plan::Key<plan::Query>,
    vs: &mut HashMap<Ident, VarState>,
    ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
    mo: &mut Option<plan::Key<plan::Operator>>,
    ctx: StreamContext,
    errs: &mut LinkedList<Diagnostic>, // TODO: thunkify or convert to loop. We mutate a passed list to make tail-call elimination easier (but rust has no TCO yet)
    con: Option<(Connector, Box<StreamExpr>)>,
) -> Result<Option<ReturnVal>, ()> {
    match (ctx, con) {
        // No more operators (no data in, or data discarded)
        (StreamContext::Nothing { last_span }, None) => Ok(None),
        (StreamContext::Continue(cont), None) => {
            discard_continue(lp, qk, cont);
            Ok(None)
        }

        // ReturnValing data from the query
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

            match build_logical(
                op,
                lp,
                tn,
                qk,
                vs,
                ts,
                mo,
                Some(Continue {
                    data_type,
                    prev_edge,
                    last_span,
                }),
            ) {
                Ok(new_ctx) => recur_stream(lp, tn, qk, vs, ts, mo, new_ctx, errs, con),
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

/// helper for extracting a map of unique fields by Ident
pub fn extract_fields<T>(
    fields: Vec<(Ident, T)>,
    err_fn: impl Fn(&Ident, &Ident) -> Diagnostic,
) -> (HashMap<Ident, T>, LinkedList<Diagnostic>) {
    let mut map_fields: HashMap<Ident, T> = HashMap::with_capacity(fields.len());
    let mut errors = LinkedList::new();
    for (id, content) in fields {
        if let Some((other_id, _)) = map_fields.get_key_value(&id) {
            errors.push_back(err_fn(&id, other_id));
        } else {
            map_fields.insert(id, content);
        }
    }

    (map_fields, errors)
}

pub fn ast_typeto_scalar(
    tn: &HashMap<Ident, plan::Key<plan::Table>>,
    ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
    t: AstType,
    table_err_fn: impl Fn(&Ident) -> Diagnostic,
    cust_err_fn: impl Fn(&Ident) -> Diagnostic,
) -> Result<plan::ScalarType, Diagnostic> {
    match t {
        AstType::RsType(t) => Ok(plan::ConcRef::Conc(plan::ScalarTypeConc::Rust(t))),
        AstType::TableRef(table_ref) => {
            if let Some(table_id) = tn.get(&table_ref) {
                Ok(plan::ConcRef::Conc(plan::ScalarTypeConc::TableRef(
                    *table_id,
                )))
            } else {
                Err(table_err_fn(&table_ref))
            }
        }
        AstType::Custom(id) => {
            if let Some(k) = ts.get(&id) {
                Ok(plan::ConcRef::Ref(*k))
            } else {
                Err(cust_err_fn(&id))
            }
        }
    }
}

/// Using a [`plan::TableAccess`] select the new records.
/// - Optionally includes fields from a previous record
/// - generates errors of invalid accesses
pub fn generate_access(
    table_id: plan::Key<plan::Table>,
    access: plan::TableAccess,
    lp: &mut plan::Plan,
    include_from: Option<plan::Key<plan::Record>>,
) -> Result<plan::Key<plan::Record>, LinkedList<Diagnostic>> {
    let new_fields = match access {
        plan::TableAccess::Ref(id) => {
            let ref_id =
                lp.scalar_types
                    .insert(plan::ConcRef::Conc(plan::ScalarTypeConc::TableRef(
                        table_id,
                    )));

            Ok(vec![(id, ref_id)])
        }
        plan::TableAccess::AllCols => {
            let table = lp.get_table(table_id);

            Ok(table
                .columns
                .iter()
                .map(|(id, plan::Column { cons, data_type })| (id.clone(), *data_type))
                .collect())
        }
        plan::TableAccess::Selection(ids) => {
            let table = lp.get_table(table_id);
            let mut fields = Vec::new();
            let mut invalid_access = LinkedList::new();
            for ColSelect { col, select_as } in ids {
                if let Some(plan::Column { cons, data_type }) = table.columns.get(&col) {
                    fields.push((select_as, *data_type));
                } else {
                    invalid_access.push_back(errors::table_query_no_such_field(&table.name, &col));
                }
            }

            if invalid_access.is_empty() {
                Ok(fields)
            } else {
                Err(invalid_access)
            }
        }
    }?;

    let mut errors = LinkedList::new();
    let mut fields = if let Some(existing) = include_from {
        lp.get_record_type(existing).fields.clone()
    } else {
        HashMap::new()
    };

    // must check even if existing is empty, a user may select the same field many times
    for (id, t) in new_fields {
        if let Some((k, t)) = fields.get_key_value(&id) {
            errors.push_back(errors::query_cannot_append_to_record(&id, k));
        } else {
            fields.insert(id, t);
        }
    }
    if errors.is_empty() {
        Ok(lp
            .record_types
            .insert(plan::ConcRef::Conc(plan::RecordConc { fields })))
    } else {
        Err(errors)
    }
}

pub fn update_incomplete(df: &mut DataFlow, to: plan::Key<plan::Operator>) {
    // TODO: determine better way to 'map' a member of an arena
    *df = match df {
        plan::DataFlow::Incomplete { from, with } => plan::DataFlow::Conn {
            from: *from,
            to,
            with: with.clone(),
        },
        plan::DataFlow::Conn { .. } | plan::DataFlow::Null => {
            unreachable!("Previous should be incomplete")
        }
    };
}

pub struct LinearBuilderState {
    pub data_out: plan::Data,
    pub op_kind: plan::OperatorKind,
    pub call_span: Span,
    pub update_mo: bool,
}

/// A helper to abstract away building linear operators and the [`Continue`].
/// - Allows construction of a function that creates an operator with an input and output.
/// - Updates the two relevant [`plan::DataFlow`]
pub fn linear_builder(
    lp: &mut plan::Plan,
    query: plan::Key<plan::Query>,
    mo: &mut Option<plan::Key<plan::Operator>>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
    op_creator: impl FnOnce(
        &mut plan::Plan,
        &Option<plan::Key<plan::Operator>>,
        Continue,
        plan::Key<plan::DataFlow>, // the next edge
    ) -> Result<LinearBuilderState, LinkedList<Diagnostic>>,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    // prev_edge -> op_key -> out_edge

    // create a new edge out of the operator
    let out_edge = lp.dataflow.insert(plan::DataFlow::Null);

    // create the operator and returned data type
    let result = op_creator(
        lp,
        mo,
        Continue {
            data_type,
            prev_edge,
            last_span,
        },
        out_edge,
    )?;
    let op_key = lp.operators.insert(plan::Operator {
        query,
        kind: result.op_kind,
    });

    if result.update_mo {
        *mo = Some(op_key);
    }

    // update the edge to contain the data out and operator key
    *lp.get_mut_dataflow(out_edge) = plan::DataFlow::Incomplete {
        from: op_key,
        with: result.data_out.clone(),
    };

    // update the edge in to contain the operator key
    update_incomplete(lp.get_mut_dataflow(prev_edge), op_key);

    Ok(StreamContext::Continue(Continue {
        data_type: result.data_out,
        prev_edge: out_edge,
        last_span: result.call_span,
    }))
}
