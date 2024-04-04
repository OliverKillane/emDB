use syn::spanned::Spanned;

use super::{Diagnostic, EMDBBackend, Ident, LinkedList, TokenStream, plan};
use crate::utils::misc::singlelist;
use quote::quote;
mod errors;
use typed_generational_arena::{StandardArena as GenArena};
use dot;

pub struct PlanViz;

impl EMDBBackend for PlanViz {
    const NAME: &'static str = "planviz";

    fn parse_options(options: Option<TokenStream>) -> Result<Self, LinkedList<Diagnostic>> {
        if let Some(opts) = options {
            Err(singlelist(errors::no_accepted_options(opts.span())))
        } else {
            Ok(PlanViz)
        }
    }
    fn generate_code(self, impl_name: Ident, plan: &plan::Plan) -> Result<TokenStream, LinkedList<Diagnostic>> {
        Ok(quote!{
            pub fn give_graph() -> &'static str {
                "hello there"
            }
        })
    }
}

#[derive(Clone)]
enum PlanNode {
    Table(plan::Key<plan::Table>),
    Operator(plan::Key<plan::Operator>),
    Dataflow(plan::Key<plan::DataFlow>),
    Query(plan::Key<plan::Query>),
}

#[derive(Clone)]
enum PlanEdge<'a> {
    DataFlow { data: plan::Key<plan::DataFlow>, flow_forward: bool, in_direction: bool },

    TableAccess { op: plan::Key<plan::Operator>, access: &'a plan::TableAccess,  table: plan::Key<plan::Table> },
    QueryReturn { op: plan::Key<plan::Operator>, query: plan::Key<plan::Query> },

    ModificationOrder { op: plan::Key<plan::Operator>, prev: plan::Key<plan::Operator> },
}

impl<'a> dot::Labeller<'a, PlanNode, PlanEdge<'a>> for plan::With<'a, Ident> {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new(self.extended.to_string()).unwrap()
    }

    fn node_id(&'a self, n: &PlanNode) -> dot::Id<'a> {
        // let (kind, name) = match n {
        //     PlanNode::Table(k) => ("Table", self.plan.get_table(k).name.to_string()),
        //     PlanNode::Operator(k) => ("")
        //     PlanNode::Dataflow(k) => 
        //     PlanNode::Query(k) => 
        // }
        todo!()
    }
}

fn get_iters<'a, T>(arena: &'a GenArena<T>, trans: impl Fn(plan::Key<T>) -> PlanNode + 'a) -> impl Iterator<Item = PlanNode> + 'a {
    arena.iter().map(move |(index, _)| trans(index))
}

impl<'a> dot::GraphWalk<'a, PlanNode, PlanEdge<'a>> for plan::With<'a, Ident> {
    fn nodes(&'a self) -> dot::Nodes<'a, PlanNode> {
        let dfs = get_iters(&self.plan.dataflow, PlanNode::Dataflow);
        let tables = get_iters(&self.plan.tables, PlanNode::Table);
        let operators = get_iters(&self.plan.operators, PlanNode::Operator);
        let queries = get_iters(&self.plan.queries, PlanNode::Query);
        dfs.chain(tables).chain(operators).chain(queries).collect()
    }

    fn edges(&'a self) -> dot::Edges<'a, PlanEdge<'a>> {
        let table_accesses = self.plan.operators.iter().filter_map(
            ||
        )
        
        let dataflows: Vec<plan::Key<plan::DataFlow>> = self.plan.dataflow.iter().map(|(index, _)| index).collect();

        // dataflows.iter().map(|index| PlanEdge::DataFlow { data: *index, flow_forward: (), in_direction: () } )



    }

    fn source(&'a self, edge: &PlanEdge<'a>) -> PlanNode {
        match edge {
            PlanEdge::IntoDataFlow { op, data, direction } => if *direction {
                PlanNode::Operator(*op)
            } else {
                PlanNode::Dataflow(*data)
            },
            PlanEdge::OutDataFlow { op, data, direction } => if *direction {
                PlanNode::Dataflow(*data)
            } else {
                PlanNode::Operator(*op)
            },
            PlanEdge::TableAccess { op, access, table } => PlanNode::Operator(*op),
            PlanEdge::QueryReturn { op, query } => PlanNode::Operator(*op),
        }
    }

    fn target(&'a self, edge: &PlanEdge<'a>) -> PlanNode {
        match edge {
            PlanEdge::IntoDataFlow { op, data, direction } => if *direction {
                PlanNode::Dataflow(*data)
            } else {
                PlanNode::Operator(*op)
            },
            PlanEdge::OutDataFlow { op, data, direction } => if *direction {
                PlanNode::Operator(*op)
            } else {
                PlanNode::Dataflow(*data)
            },
            PlanEdge::TableAccess { op, access, table } => PlanNode::Table(*table),
            PlanEdge::QueryReturn { op, query } => PlanNode::Query(*query),
        }
    }
}
