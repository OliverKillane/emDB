use std::collections::{HashMap, HashSet, LinkedList};

use proc_macro2::{Span, Ident};
use proc_macro_error::Diagnostic;

use crate::{plan::repr::{EdgeKey, LogicalPlan, LogicalQuery, LogicalQueryParams, QueryKey, Record, TableKey}, utils::misc::singlelist};

use super::ast::{FuncOp, Operator, Query, StreamExpr, Table, Connector};



fn ast_to_logical() -> Result<LogicalPlan, LinkedList<Diagnostic>> {
    todo!()
}

fn add_table(
    lp: &mut LogicalPlan,
    Table {
        name,
        cols,
        cons,
    }: Table,
) -> Result<(Ident, TableKey), LinkedList<Diagnostic>> {
    todo!()
}

enum StreamContext {
    Returned { 
        span: Span, 
        index: EdgeKey 
    },
    Continue {
        data_type: Record,
        prev_edge: EdgeKey,
        last_span: Span,
    }
}

enum VarState {
    Used(Span),
    Available(StreamContext)
}

fn add_query(
    lp: &mut LogicalPlan,
    qs: &mut HashSet<Ident>,
    tn: &HashMap<Ident, TableKey>,
    Query{ name, params, streams } : Query, 
) -> Option<LinkedList<Diagnostic>> {
    let params = params.into_iter().map(|(name, data_type)| LogicalQueryParams{ name, data_type}).collect();
    let qk = lp.queries.insert(LogicalQuery { name, params, returnval: None });
    let mut vs = HashMap::new();
    
    // return twice?

    for stream in streams {
        build_streamexpr(lp, tn, qk, &mut vs, stream);
    }
    todo!()
}

fn build_streamexpr(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    StreamExpr { op, con }: StreamExpr,
) -> Option<LinkedList<Diagnostic>> {
    match build_op_start(lp, tn, qk, vs, op) {
        Ok(ctx) => recur_stream(lp, tn, qk, vs, Some(ctx), con),
        Err(es) => Some(es),
    }
}

fn recur_stream(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    ctx: Option<StreamContext>, 
    con: Option<(Connector, Box<StreamExpr>)>
) -> Result<Option<EdgeKey>, LinkedList<Diagnostic>> {
    match (ctx, con) {
        (Some(ctx), Some((conn, e))) => {
            
            todo!()
        },
        (None, None) => todo!(),
        (None, Some(_)) => todo!(),
        (Some(_), None) => todo!(),
    }
}

fn build_op_start(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &HashMap<Ident, VarState>,
    op: Operator,
) -> Result<StreamContext, LinkedList<Diagnostic>> {
    match op {
        Operator::Ref { ref_span, table_name } => todo!(),
        Operator::Use { use_span, var_name } => todo!(),
        Operator::FuncOp { fn_span, op: FuncOp::Unique { table, refs, unique_field, from_expr } } => todo!(),
        
        Operator::FuncOp { fn_span, op } => todo!(),
        Operator::Ret { ret_span } => todo!(),
        Operator::Let { let_span, var_name } => todo!(),
    }
}

fn build_op_continue(
    lp: &mut LogicalPlan,
    tn: &HashMap<Ident, TableKey>,
    qk: QueryKey,
    vs: &mut HashMap<Ident, VarState>,
    prev: StreamContext,
    curr_op: Operator,
) -> Result<Option<StreamContext>, LinkedList<Diagnostic>> {
    match curr_op {
        Operator::Ref { ref_span, table_name } => todo!(),
        Operator::Use { use_span, var_name } => todo!(),
        Operator::FuncOp { fn_span, op: FuncOp::Unique { table, refs, unique_field, from_expr } } => todo!(),
        
        Operator::FuncOp { fn_span, op } => todo!(),
        Operator::Ret { ret_span } => todo!(),
        Operator::Let { let_span, var_name } => todo!(),
    }
}