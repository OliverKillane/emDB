use std::{fs::File, path::Path};

use combi::{core::{mapsuc, seq}, macros::seqs, tokens::{basic::{collectuntil, isempty, matchident, matchpunct, syn}, error::expectederr, TokenDiagnostic, TokenIter}, Combi, Repr};
use syn::LitStr;
use super::{Diagnostic, EMDBBackend, Ident, LinkedList, TokenStream, plan};
use crate::utils::misc::singlelist;
use quote::quote;
use typed_generational_arena::{StandardArena as GenArena};
use dot;

mod errors;
mod edges;
mod nodes;

use edges::{GetEdges, PlanEdge, get_dataflow};
use nodes::{PlanNode, StyleableNode};

pub struct PlanViz {
    out_location: LitStr
}

impl EMDBBackend for PlanViz {
    const NAME: &'static str = "planviz";

    fn parse_options(backend_name: &Ident, options: Option<TokenStream>) -> Result<Self, LinkedList<Diagnostic>> {
        let parser = expectederr(mapsuc(
            seqs!(
                matchident("path"),
                matchpunct('='),
                syn(collectuntil(isempty()))
            ),
            |(_, (_, out_location)): (_, (_, LitStr))| PlanViz{ out_location } 
        ));
        if let Some(opts) = options {
            let (_, res) = parser.comp(TokenIter::from(opts, backend_name.span()));
            res.to_result().map_err(TokenDiagnostic::into_list)
        } else {
            Err(singlelist(errors::expected_options(backend_name, &format!("{}", Repr(&parser)))))
        }
    }

    fn generate_code(self, impl_name: Ident, plan: &plan::Plan) -> Result<TokenStream, LinkedList<Diagnostic>> {
        let out_path_str = self.out_location.value();
        match File::create_new(Path::new(&out_path_str)) {
            Ok(mut open_file) => {
                match dot::render(&plan::With { plan, extended: impl_name.clone() }, &mut open_file) {
                    Ok(()) => { Ok(quote! {
mod #impl_name {
    pub const OUT_DIRECTORY: &str = #out_path_str;
}
                    }) }
                    Err(e) => return Err(singlelist(errors::io_error(&impl_name, self.out_location.span(), &e)))
                }
            },
            Err(e) => {
                let span = self.out_location.span();
                Err(singlelist(errors::io_error(&impl_name, span, &e)))
            }
        }
    }
}

// NOTE: complex but eliminates boilerplate
//       - match by query type, then apply the method
//       - variable names need to be passed (hygenic macro)
macro_rules! node_call {
    (match $self_id:ident , $node_id:ident -> $it:ident, $key:ident => $($tk:tt)*) => {
        match $node_id {
            PlanNode::Table($key) => {let $it = $self_id.plan.get_table(*$key); $($tk)* },
            PlanNode::Operator($key) => {let $it = $self_id.plan.get_operator(*$key); $($tk)* },
            PlanNode::Dataflow($key) => {let $it = $self_id.plan.get_dataflow(*$key); $($tk)* },
            PlanNode::Query($key) => {let $it = $self_id.plan.get_query(*$key); $($tk)* },
        }
    }
}

impl<'a> dot::Labeller<'a, PlanNode, PlanEdge> for plan::With<'a, Ident> {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new(self.extended.to_string()).unwrap()
    }

    fn node_id(&'a self, n: &PlanNode) -> dot::Id<'a> {
        node_call!(match self, n -> it, key => it.id(*key))
    }

    fn node_shape(&'a self, n: &PlanNode) -> Option<dot::LabelText<'a>> {
        node_call!(match self, n -> it, key => it.shape(self.plan))
    }

    fn node_label(&'a self, n: &PlanNode) -> dot::LabelText<'a> {
        node_call!(match self, n -> it, key => it.label(self.plan))
    }

    fn node_style(&'a self, n: &PlanNode) -> dot::Style {
        node_call!(match self, n -> it, key => it.style(self.plan))
    }

    fn node_color(&'a self, n: &PlanNode) -> Option<dot::LabelText<'a>> {
        node_call!(match self, n -> it, key => it.color(self.plan))
    }
}

fn get_iters<'a, T>(arena: &'a GenArena<T>, trans: impl Fn(plan::Key<T>) -> PlanNode + 'a) -> impl Iterator<Item = PlanNode> + 'a {
    arena.iter().map(move |(index, _)| trans(index))
}

impl<'a> dot::GraphWalk<'a, PlanNode, PlanEdge> for plan::With<'a, Ident> {
    fn nodes(&'a self) -> dot::Nodes<'a, PlanNode> {
        let dfs = get_iters(&self.plan.dataflow, PlanNode::Dataflow);
        let tables = get_iters(&self.plan.tables, PlanNode::Table);
        let operators = get_iters(&self.plan.operators, PlanNode::Operator);
        let queries = get_iters(&self.plan.queries, PlanNode::Query);
        dfs.chain(tables).chain(operators).chain(queries).collect()
    }

    fn edges(&'a self) -> dot::Edges<'a, PlanEdge> {
        // TODO: use coroutines/generators to remove the need for multiple iters over operators and dataflow.
        //       - coroutines are currently on nightly, but not stable
        //       - libraries like [remit](https://docs.rs/remit/latest/remit/) can be used, but I want to
        //         reduce dependencies
        let mut edges = Vec::new();
        for (index, dataflow) in self.plan.dataflow.iter() {
            dataflow.get_edges(index, &mut edges);
        }
        for (index, operator) in self.plan.operators.iter() {
            operator.get_edges(index, &mut edges);
        }
        edges.into()
    }

    fn source(&'a self, edge: &PlanEdge) -> PlanNode {
        match edge {
            PlanEdge::DataFlow { data, flow_forward, to_direction } => get_dataflow(*data, self.plan.get_dataflow(*data), *flow_forward, *to_direction, true),
            PlanEdge::TableAccess { op, .. } | PlanEdge::QueryReturn { op, .. } | PlanEdge::ModificationOrder { op, .. } => PlanNode::Operator(*op),
        }
    }

    fn target(&'a self, edge: &PlanEdge) -> PlanNode {
        match edge {
            PlanEdge::DataFlow { data, flow_forward, to_direction } => get_dataflow(*data, self.plan.get_dataflow(*data), *flow_forward, *to_direction, false),
            PlanEdge::TableAccess { op, table } => PlanNode::Table(*table),
            PlanEdge::QueryReturn { op, query } => PlanNode::Query(*query),
            PlanEdge::ModificationOrder { op, prev } => PlanNode::Operator(*prev),
        }
    }
}