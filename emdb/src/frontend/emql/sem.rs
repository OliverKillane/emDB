use crate::{
    backend::{BackendTypes, GraphViz, Simple},
    frontend::emql::{
        ast::{
            Ast, AstType, BackendImpl, BackendKind, Connector, Constraint, ConstraintExpr, Query,
            StreamExpr, Table,
        },
        errors,
    },
    plan::repr::{
        Record, EdgeKey, LogicalColumn, LogicalColumnConstraint, LogicalPlan, LogicalQuery, LogicalQueryParams, LogicalRowConstraint, LogicalTable, OpKey, QueryKey, ScalarType, TableKey, UniqueCons
    },
};
use proc_macro2::{Ident, Span};
use proc_macro_error::Diagnostic;
use std::collections::{HashMap, HashSet, LinkedList};

use super::operators::{build_logical};

pub(super) struct ReturnVal {
    pub span: Span,
    pub index: OpKey,
}

#[derive(Clone)]
pub(super) struct Continue {
    pub data_type: Record,
    pub prev_edge: EdgeKey,
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
            let dt = match data_type {
                AstType::RsType(t) => Some(ScalarType::Rust(t)),
                AstType::TableRef(table_ref) => {
                    if let Some(table_id) = tn.get(&table_ref) {
                        Some(ScalarType::Ref(*table_id))
                    } else {
                        errors
                            .push_back(errors::query_param_ref_table_not_found(&name, &table_ref));
                        None
                    }
                }
            }?;

            Some(LogicalQueryParams {
                name,
                data_type: dt,
            })
        })
        .collect();

    let qk = lp.queries.insert(LogicalQuery {
        name: name.clone(),
        params,
        returnval: None,
    });
    let mut vs = HashMap::new();
    let mut ret: Option<ReturnVal> = None;

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

fn build_streamexpr(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    StreamExpr { op, con }: StreamExpr,
) -> Result<Option<ReturnVal>, LinkedList<Diagnostic>> {
    match build_logical(op, lp, tn, qk, vs, None) {
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
                Some(Continue {
                    data_type,
                    prev_edge,
                    last_span,
                }),
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
