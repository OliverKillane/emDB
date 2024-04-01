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
    plan,
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
) -> Result<(plan::LogicalPlan, backend::Targets), LinkedList<Diagnostic>> {
    let mut errors = LinkedList::new();
    let mut lp = plan::LogicalPlan::new();
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
    lp: &mut plan::LogicalPlan,
    tn: &mut HashMap<Ident, plan::Key<plan::Table>>,
    Table { name, cols, cons }: Table,
) -> LinkedList<Diagnostic> {
    let mut errs = LinkedList::new();

    // Add each column, checking for duplicate names
    let mut columns: HashMap<Ident, plan::Column> = HashMap::new();
    for (col_name, col_type) in cols {
        match columns.get_key_value(&col_name) {
            Some((duplicate, _)) => {
                errs.push_back(errors::table_column_redefined(&col_name, duplicate))
            }
            None => {
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
                        &alias,
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
    lp: &mut plan::LogicalPlan,
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
    errors
}

fn build_streamexpr(
    lp: &mut plan::LogicalPlan,
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

fn recur_stream(
    lp: &mut plan::LogicalPlan,
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
        (StreamContext::Continue { .. }, None) => Ok(None),

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
