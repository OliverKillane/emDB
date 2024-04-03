use syn::spanned::Spanned;

use super::{Diagnostic, EMDBBackend, Ident, LinkedList, TokenStream, plan};
use crate::utils::misc::singlelist;
use quote::quote;
mod errors;

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

// enum LogicalNode {
//     Table(TableKey),
//     Operator(OpKey),
//     Query(QueryKey),
// }

// impl<'a> dot::Labeller<'a, Nd, Ed> for Edges {
//     fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example1").unwrap() }
//
//     fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
//         dot::Id::new(format!("N{}", *n)).unwrap()
//     }
// }
//
// impl<'a> dot::GraphWalk<'a, Nd, Ed> for Edges {
//     fn nodes(&self) -> dot::Nodes<'a,Nd> {
//         // (assumes that |N| \approxeq |E|)
//         let &Edges(ref v) = self;
//         let mut nodes = Vec::with_capacity(v.len());
//         for &(s,t) in v {
//             nodes.push(s); nodes.push(t);
//         }
//         nodes.sort();
//         nodes.dedup();
//         Cow::Owned(nodes)
//     }
//
//     fn edges(&'a self) -> dot::Edges<'a,Ed> {
//         let &Edges(ref edges) = self;
//         Cow::Borrowed(&edges[..])
//     }
//
//     fn source(&self, e: &Ed) -> Nd { e.0 }
//
//     fn target(&self, e: &Ed) -> Nd { e.1 }
// }
