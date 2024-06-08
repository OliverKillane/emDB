//! # Semantic Analysis
//! ## Structure
//! This module contains the translation from [`super::ast`] to [`crate::plan`].
//!
//! Queries, Tables and Backends are added by mutating an initially empty plan.
//! - Tracking of emql concepts such as types & type aliases are managed here.
//! - Complex or reused analysis for the operators is included in theis module.

use super::{ast::Context, operators::build_logical};
use crate::{
    backend,
    frontend::emql::{
        ast::{
            Ast, AstType, BackendImpl, Connector, Constraint, ConstraintExpr, Query, StreamExpr,
            Table,
        },
        errors,
    },
    plan::{self, ConcRef, ScalarTypeConc},
};
use proc_macro2::{Ident, Span};
use proc_macro_error::Diagnostic;
use std::collections::{HashMap, HashSet, LinkedList};

/// The return value of a [`Context`], including the [`ReturnVal::span`] for multiple
/// return errors.
pub(super) struct ReturnVal {
    pub span: Span,
    pub index: plan::Key<plan::Operator>,
}

/// The output stream from a [`plan::Operator`] to allow it to be used by the
/// analysis for the next operator.
#[derive(Clone)]
pub(super) struct Continue {
    pub data_type: plan::Data,
    pub prev_edge: plan::Key<plan::DataFlow>,
    pub last_span: Span,
}

/// The possible outputs for an operator
pub(super) enum StreamContext {
    /// The operator consumes, but no output is provided
    Nothing { last_span: Span },
    /// [`plan::Return`] operators
    Returned(ReturnVal),
    /// Any [`plan::Operator`] with a single output.
    Continue(Continue),
}

/// State of en emql variable, can only be created, and used once.
pub(super) enum VarState {
    Used { created: Span, used: Span },
    Available { created: Span, state: Continue },
}

/// Convert an [`Ast`] to a [`plan::Plan`] and [`backend::Targets`]
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

/// Add a new backend
/// - Each has a unique name
/// - Each backend needs to parse its own options
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

/// Add a table to a [`plan::Plan`]
/// - Table must be unique
/// - Constraint names must be unique
/// - Some constraints are not redefinable (e.g. unique cannot be used multiple
///   times of one column)
fn add_table(
    lp: &mut plan::Plan,
    tn: &mut HashMap<Ident, plan::Key<plan::Table>>,
    Table { name, cols, cons }: Table,
) -> LinkedList<Diagnostic> {
    let mut errs = LinkedList::new();

    // Add each column, checking for duplicate names
    let mut columns: HashMap<plan::RecordField, plan::Column> = HashMap::new();
    for (col_name, col_type) in cols {
        let col_rf = col_name.clone().into();
        if let Some((duplicate, _)) = columns.get_key_value(&col_rf) {
            // INV: we can only conflict with other user defined fields
            errs.push_back(errors::table_column_redefined(
                &col_name,
                duplicate.get_field(),
            ));
        } else {
            let type_index =
                lp.scalar_types
                    .insert(plan::ConcRef::Conc(plan::ScalarTypeConc::Rust {
                        type_context: plan::TypeContext::DataStore,
                        ty: col_type,
                    }));
            columns.insert(
                col_rf,
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
        if let Some(duplicate) = constraint_names.get(&alias) {
            errs.push_back(errors::table_constraint_alias_redefined(&alias, duplicate));
        } else {
            constraint_names.insert(alias.clone());
        }

        match expr {
            ConstraintExpr::Unique { field } => {
                let rf_field = field.clone().into();
                match columns.get_mut(&rf_field) {
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
                }
            }
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

/// Add a query to a [`plan::Plan`], using [`add_streams_to_context`].
fn add_query(
    lp: &mut plan::Plan,
    qs: &mut HashSet<Ident>,
    tn: &HashMap<Ident, plan::Key<plan::Table>>,
    Query {
        name,
        context: Context { params, streams },
    }: Query,
) -> LinkedList<Diagnostic> {
    let mut vs = HashMap::new();
    let mut ts = HashMap::new();

    // Analyse the query parameters
    let (raw_params, mut errors) =
        extract_fields_ordered(params, errors::query_parameter_redefined);
    let params =
        raw_params.into_iter().filter_map(|(name, data_type)| {
            match query_ast_typeto_scalar(
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
        });

    // Create and populate the query context
    let op_ctx = lp
        .contexts
        .insert(plan::Context::from_params(params.collect(), Vec::new()));
    lp.queries.insert(plan::Query {
        name: name.clone(),
        ctx: op_ctx,
    });
    add_streams_to_context(
        lp,
        tn,
        &mut ts,
        &mut vs,
        op_ctx,
        streams,
        &name,
        &mut errors,
    );

    // discard the unused variables of the context
    discard_ends(lp, op_ctx, vs);

    errors
}

/// Add a collection of streams to a context (e.g. a [`Query`], or the inside of
/// a [`plan::ForEach`])
#[allow(clippy::too_many_arguments)]
pub fn add_streams_to_context(
    lp: &mut plan::Plan,
    tn: &HashMap<Ident, plan::Key<plan::Table>>,

    // types are per-query, contexts can introduce new types to outer, outer passes through to inner
    ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,

    // each context gets its own variables, including being able to use some provided (e.g. [`plan::Foreach`], [`plan::GroupBy`] streams)
    vs: &mut HashMap<Ident, VarState>,

    op_ctx: plan::Key<plan::Context>,
    streams: Vec<StreamExpr>,
    context_name: &Ident,

    errors: &mut LinkedList<Diagnostic>,
) {
    let mut ret: Option<ReturnVal> = None;

    for stream in streams {
        match build_streamexpr(lp, tn, vs, ts, op_ctx, stream) {
            Ok(r) => match (&mut ret, r) {
                (Some(prev_ret), Some(new_ret)) => {
                    errors.push_back(errors::query_multiple_returns(
                        new_ret.span,
                        prev_ret.span,
                        context_name,
                    ));
                }
                (Some(_), None) => (),
                (None, rs) => ret = rs,
            },
            Err(mut es) => errors.append(&mut es),
        }
    }

    if let Some(ret) = ret {
        lp.contexts[op_ctx].set_return(ret.index);
    }
}

/// Discard the unused variables (using [`plan::Discard`])
pub fn discard_ends(
    lp: &mut plan::Plan,
    op_ctx: plan::Key<plan::Context>,
    vs: HashMap<Ident, VarState>,
) {
    for (name, vs) in vs {
        if let VarState::Available { created, state } = vs {
            discard_continue(lp, op_ctx, state);
        }
    }
}

fn build_streamexpr(
    lp: &mut plan::Plan,
    tn: &HashMap<Ident, plan::Key<plan::Table>>,
    vs: &mut HashMap<Ident, VarState>,
    ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
    op_ctx: plan::Key<plan::Context>,
    StreamExpr { op, con }: StreamExpr,
) -> Result<Option<ReturnVal>, LinkedList<Diagnostic>> {
    match build_logical(op, lp, tn, vs, ts, op_ctx, None) {
        Ok(ctx) => {
            let mut errors = LinkedList::new();
            match recur_stream(lp, tn, vs, ts, op_ctx, ctx, &mut errors, con) {
                Ok(res) if errors.is_empty() => Ok(res),
                _ => Err(errors),
            }
        }
        Err(es) => Err(es),
    }
}

fn discard_continue(
    lp: &mut plan::Plan,
    ctx: plan::Key<plan::Context>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
) {
    let discard_op = lp
        .operators
        .insert(plan::Discard { input: prev_edge }.into());
    update_incomplete(lp.get_mut_dataflow(prev_edge), discard_op);
    lp.get_mut_context(ctx).add_discard(discard_op);
}

#[allow(clippy::too_many_arguments)]
fn recur_stream(
    lp: &mut plan::Plan,
    tn: &HashMap<Ident, plan::Key<plan::Table>>,
    vs: &mut HashMap<Ident, VarState>,
    ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
    op_ctx: plan::Key<plan::Context>,
    ctx: StreamContext,
    errs: &mut LinkedList<Diagnostic>, // TODO: thunkify or convert to loop. We mutate a passed list to make tail-call elimination easier (but rust has no TCO yet)
    con: Option<(Connector, Box<StreamExpr>)>,
) -> Result<Option<ReturnVal>, ()> {
    match (ctx, con) {
        // No more operators (no data in, or data discarded)
        (StreamContext::Nothing { last_span }, None) => Ok(None),
        (StreamContext::Continue(cont), None) => {
            discard_continue(lp, op_ctx, cont);
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
                vs,
                ts,
                op_ctx,
                Some(Continue {
                    data_type,
                    prev_edge,
                    last_span,
                }),
            ) {
                Ok(new_ctx) => recur_stream(lp, tn, vs, ts, op_ctx, new_ctx, errs, con),
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

/// Creates a hashmap from the fields, with errors for duplicate fields
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

/// Similar to [extract_fields] but maintains the order of fields from the original vector.
/// - duplicates of fields are removed
/// - errors are generated for each duplicate
pub fn extract_fields_ordered<T>(
    fields: Vec<(Ident, T)>,
    err_fn: impl Fn(&Ident, &Ident) -> Diagnostic,
) -> (Vec<(Ident, T)>, LinkedList<Diagnostic>) {
    let mut errors = LinkedList::new();
    let mut used_names = HashSet::with_capacity(fields.len());

    let non_dup_fields = fields
        .into_iter()
        .filter_map(|(id, content)| {
            if let Some(other_id) = used_names.get(&id) {
                errors.push_back(err_fn(&id, other_id));
                None
            } else {
                used_names.insert(id.clone());
                Some((id, content))
            }
        })
        .collect();

    (non_dup_fields, errors)
}

/// Converts an AST type to a scalar type, if a rust type then the [`plan::TypeContext::Query`] context is used.
pub fn query_ast_typeto_scalar(
    tn: &HashMap<Ident, plan::Key<plan::Table>>,
    ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
    t: AstType,
    table_err_fn: impl Fn(&Ident) -> Diagnostic,
    cust_err_fn: impl Fn(&Ident) -> Diagnostic,
) -> Result<plan::ScalarType, Diagnostic> {
    match t {
        AstType::RsType(ty) => Ok(plan::ConcRef::Conc(plan::ScalarTypeConc::Rust {
            type_context: plan::TypeContext::Query,
            ty,
        })),
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

impl plan::RecordField {
    /// Used for conflicts, where this *should* only possible between user generated types
    pub fn get_field(&self) -> &Ident {
        match self {
            plan::RecordField::User(id) => id,
            plan::RecordField::Internal(_) => {
                unreachable!("Attempted to get field, but was an internal id, this is a bug")
            }
        }
    }
}

fn append_fields(
    lp: &mut plan::Plan,
    new_fields: Vec<(Ident, plan::Key<plan::ScalarType>)>,
    existing_fields: plan::Key<plan::RecordType>,
) -> Result<plan::Key<plan::RecordType>, LinkedList<Diagnostic>> {
    let mut errors = LinkedList::new();
    let mut fields = lp.get_record_type_conc(existing_fields).fields.clone();

    // must check even if existing is empty, a user may select the same field many times
    for (id, t) in new_fields {
        let rec_id = id.into();
        if let Some((k, t)) = fields.get_key_value(&rec_id) {
            errors.push_back(errors::query_cannot_append_to_record(
                rec_id.get_field(),
                k.get_field(),
            ));
        } else {
            fields.insert(rec_id, t);
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

/// Generate a node that scans references from a given table
/// Used in both `ref .. as ..` and `use ...`
pub fn create_scanref(
    lp: &mut plan::Plan,
    op_ctx: plan::Key<plan::Context>,
    table_id: plan::Key<plan::Table>,
    out_ref: plan::RecordField,
    call_span: Span,
) -> Continue {
    let record_out = plan::Data {
        fields: generate_access::reference(table_id, out_ref.clone(), lp),
        stream: true,
    };
    let out_edge = lp.dataflow.insert(plan::DataFlow::Null);
    let ref_op = lp.operators.insert(
        plan::ScanRefs {
            table: table_id,
            out_ref,
            output: out_edge,
        }
        .into(),
    );

    *lp.get_mut_dataflow(out_edge) = plan::DataFlow::Incomplete {
        from: ref_op,
        with: record_out.clone(),
    };

    lp.get_mut_context(op_ctx).add_operator(ref_op);

    Continue {
        data_type: record_out,
        prev_edge: out_edge,
        last_span: call_span,
    }
}

pub struct FieldComparison<'res> {
    pub extra_fields: Vec<&'res Ident>,
    pub missing_fields: Vec<&'res Ident>,
}

/// Check if the user defined fields present match a data type.
/// INV: the data type has no [`plan::RecordField::Internal`] fields
pub fn check_fields_type<'imm>(
    lp: &'imm plan::Plan,
    data_type: plan::Key<plan::RecordType>,
    fields: impl Iterator<Item = &'imm Ident>,
) -> FieldComparison<'imm> {
    let mut keys = lp
        .get_record_type_conc(data_type)
        .fields
        .keys()
        .map(|rf| match rf {
            plan::RecordField::User(i) => i,
            plan::RecordField::Internal(_) => {
                unreachable!("Cannot call this method with internal fields")
            }
        })
        .collect::<HashSet<_>>();

    let mut extra_fields = Vec::new();
    let mut missing_fields = Vec::new();

    for field in fields {
        if !keys.remove(field) {
            extra_fields.push(field);
        }
    }

    for field in keys {
        missing_fields.push(field);
    }

    FieldComparison {
        extra_fields,
        missing_fields,
    }
}

pub fn get_all_cols(lp: &mut plan::Plan, table_id: plan::Key<plan::Table>) -> plan::RecordConc {
    // NOTE: cannot use lp.get_table as borrow checker does not know that does
    //       not borrow from lp.scalar_types which is mutated later
    let table = lp.tables.get(table_id).unwrap();
    plan::RecordConc {
        fields: table
            .columns
            .iter()
            .map(|(id, plan::Column { cons, data_type })| {
                (id.clone(), {
                    let access = ConcRef::Conc(ScalarTypeConc::TableGet {
                        table: table_id,
                        field: id.clone(),
                    });
                    lp.scalar_types.insert(access)
                })
            })
            .collect(),
    }
}

pub fn insert_all_cols(lp: &plan::Plan, table_id: plan::Key<plan::Table>) -> plan::RecordConc {
    let table = lp.get_table(table_id);
    plan::RecordConc {
        fields: table
            .columns
            .iter()
            .map(|(id, plan::Column { cons, data_type })| (id.clone(), *data_type))
            .collect(),
    }
}

/// Generating the types for different kinds of accesses, as used in the access
/// operators such as [`plan::UniqueRef`] or [`plan::DeRef`]
pub mod generate_access {
    use super::*;

    pub fn reference(
        table_id: plan::Key<plan::Table>,
        id: plan::RecordField,
        lp: &mut plan::Plan,
    ) -> plan::Key<plan::RecordType> {
        let ref_id = lp
            .scalar_types
            .insert(plan::ConcRef::Conc(plan::ScalarTypeConc::TableRef(
                table_id,
            )));
        lp.record_types.insert(
            plan::RecordConc {
                fields: HashMap::from([(id, ref_id)]),
            }
            .into(),
        )
    }

    pub struct DereferenceTypes {
        pub outer_record: plan::Key<plan::RecordType>,
        pub inner_record: plan::Key<plan::RecordType>,
    }

    pub fn dereference(
        table_id: plan::Key<plan::Table>,
        lp: &mut plan::Plan,
        new_field: Ident,
        include_from: plan::Key<plan::RecordType>,
    ) -> Result<DereferenceTypes, LinkedList<Diagnostic>> {
        let cols = get_all_cols(lp, table_id);
        let inner_record = lp.record_types.insert(cols.into());
        let scalar_t = lp
            .scalar_types
            .insert(plan::ConcRef::Conc(plan::ScalarTypeConc::Record(
                inner_record,
            )));
        let outer_record = append_fields(lp, vec![(new_field, scalar_t)], include_from)?;
        Ok(DereferenceTypes {
            outer_record,
            inner_record,
        })
    }

    pub fn insert(
        table_id: plan::Key<plan::Table>,
        lp: &mut plan::Plan,
    ) -> plan::Key<plan::RecordType> {
        lp.record_types.insert(insert_all_cols(lp, table_id).into())
    }

    pub fn unique(
        table_id: plan::Key<plan::Table>,
        id: Ident,
        lp: &mut plan::Plan,
        include_from: plan::Key<plan::RecordType>,
    ) -> Result<plan::Key<plan::RecordType>, LinkedList<Diagnostic>> {
        let ref_id = lp
            .scalar_types
            .insert(plan::ConcRef::Conc(plan::ScalarTypeConc::TableRef(
                table_id,
            )));
        append_fields(lp, vec![(id, ref_id)], include_from)
    }
}

pub fn update_incomplete(df: &mut plan::DataFlow, to: plan::Key<plan::Operator>) {
    // TODO: determine better way to 'map' a member of an arena
    *df = match df {
        plan::DataFlow::Incomplete { from, with } => plan::DataFlow::Conn(plan::DataFlowConn {
            from: *from,
            to,
            with: with.clone(),
        }),
        plan::DataFlow::Conn { .. } | plan::DataFlow::Null => {
            unreachable!("Previous should be incomplete")
        }
    };
}

pub fn get_user_fields(rec: &plan::RecordConc) -> Vec<&Ident> {
    rec.fields
        .iter()
        .filter_map(|(k, _)| {
            if let plan::RecordField::User(id) = k {
                Some(id)
            } else {
                None
            }
        })
        .collect()
}

pub struct LinearBuilderState {
    pub data_out: plan::Data,
    pub op: plan::Operator,
    pub call_span: Span,
}

/// A helper to abstract away building linear operators and the [`Continue`].
/// - Allows construction of a function that creates an operator with an input and output.
/// - Updates the two relevant [`plan::DataFlow`]
pub fn linear_builder(
    lp: &mut plan::Plan,
    op_ctx: plan::Key<plan::Context>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
    op_creator: impl FnOnce(
        &mut plan::Plan,
        plan::Key<plan::Context>,
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
        op_ctx,
        Continue {
            data_type,
            prev_edge,
            last_span,
        },
        out_edge,
    )?;
    let op_key = lp.operators.insert(result.op);

    lp.get_mut_context(op_ctx).add_operator(op_key);

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

/// Like [`linear_builder`] but for use internally, when semantic errors are not
/// possible.
pub fn valid_linear_builder(
    lp: &mut plan::Plan,
    op_ctx: plan::Key<plan::Context>,
    Continue {
        data_type,
        prev_edge,
        last_span,
    }: Continue,
    op_creator: impl FnOnce(
        &mut plan::Plan,
        plan::Key<plan::Context>,
        Continue,
        plan::Key<plan::DataFlow>, // the next edge
    ) -> LinearBuilderState,
) -> Continue {
    // prev_edge -> op_key -> out_edge

    // create a new edge out of the operator
    let out_edge = lp.dataflow.insert(plan::DataFlow::Null);

    // create the operator and returned data type
    let result = op_creator(
        lp,
        op_ctx,
        Continue {
            data_type,
            prev_edge,
            last_span,
        },
        out_edge,
    );
    let op_key = lp.operators.insert(result.op);

    lp.get_mut_context(op_ctx).add_operator(op_key);

    // update the edge to contain the data out and operator key
    *lp.get_mut_dataflow(out_edge) = plan::DataFlow::Incomplete {
        from: op_key,
        with: result.data_out.clone(),
    };

    // update the edge in to contain the operator key
    update_incomplete(lp.get_mut_dataflow(prev_edge), op_key);

    Continue {
        data_type: result.data_out,
        prev_edge: out_edge,
        last_span: result.call_span,
    }
}
