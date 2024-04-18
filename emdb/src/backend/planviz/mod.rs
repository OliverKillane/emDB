use std::{fs::File, path::Path};

use combi::{core::{choice, mapsuc, seq, setrepr}, macros::{choices, seqs}, tokens::{basic::{collectuntil, gettoken, matchident, matchpunct, peekident, peekpunct, syn}, error::{error, expectederr}, TokenDiagnostic, TokenIter, TokenParser}, Combi, Repr};
use syn::LitStr;
use super::{EMDBBackend, Ident, LinkedList, TokenStream, plan};
use proc_macro_error::{Diagnostic, Level};
use crate::utils::misc::singlelist;
use quote::quote;
use typed_generational_arena::{StandardArena as GenArena};
use dot;

mod errors;
mod edges;
mod nodes;

use edges::{PlanEdge, EdgeStyle, get_edges};
use nodes::{PlanNode, StyleableNode, node_call, get_nodes};

pub struct PlanViz {
    out_location: LitStr,
    config: DisplayConfig
}

pub struct DisplayConfig {
    display_types: bool,
    display_ctx_ops: bool,
    display_control: bool
}

impl EMDBBackend for PlanViz {
    const NAME: &'static str = "Planviz";

    fn parse_options(backend_name: &Ident, options: Option<TokenStream>) -> Result<Self, LinkedList<Diagnostic>> {
        fn on_off(name: &'static str) -> impl TokenParser<bool> {
            mapsuc(seqs!(
                matchpunct(','),
                matchident(name),
                matchpunct('='),
                choices!(
                    peekident("on") => mapsuc(matchident("on"), |_| true),
                    peekident("off") => mapsuc(matchident("off"), |_| false),
                    otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, "Expected `on` or `off`".to_owned()))
                )
            ), |(_, (_, (_, opt)))| opt)
        }
        let parser = expectederr(mapsuc(
            expectederr(seqs!(
                matchident("path"),
                matchpunct('='),
                setrepr(syn(collectuntil(peekpunct(','))), "<file path>"),
                on_off("display_types"),
                on_off("display_ctx_ops"),
                on_off("display_control")
            )),
            |(_, (_, (out_location, (display_types, (display_ctx_ops, display_control))))): (_, (_, (LitStr, _)))| PlanViz{ out_location, config: DisplayConfig{display_types,display_ctx_ops, display_control} } 
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
        match File::create(Path::new(&out_path_str)) {
            Ok(mut open_file) => {
                match dot::render(&plan::With { plan, extended: (impl_name.clone(), self.config) }, &mut open_file) {
                    Ok(()) => { Ok(quote! {
mod #impl_name {
    pub const OUT_DIRECTORY: &str = #out_path_str;
}
                    }) }
                    Err(e) => Err(singlelist(errors::io_error(&impl_name, self.out_location.span(), &e)))
                }
            },
            Err(e) => {
                let span = self.out_location.span();
                Err(singlelist(errors::io_error(&impl_name, span, &e)))
            }
        }
    }
}

trait GetFeature<T>: Sized {
    fn get_all(edges: &mut Vec<T>, arena: &GenArena<Self>, config: &DisplayConfig) {
        for (key, node) in arena.iter() {
            node.get_features(key, edges, config);
        }
    }
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<T>, config: &DisplayConfig);
}

impl<'a> dot::Labeller<'a, PlanNode, PlanEdge> for plan::With<'a, (Ident, DisplayConfig)> {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new(self.extended.0.to_string()).unwrap()
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

    fn edge_label(&'a self, e: &PlanEdge) -> dot::LabelText<'a> {
        e.label()
    }

    fn edge_end_arrow(&'a self, e: &PlanEdge) -> dot::Arrow {
        e.end_arrow()
    }

    fn edge_start_arrow(&'a self, e: &PlanEdge) -> dot::Arrow {
        e.start_arrow()
    }

    fn edge_style(&'a self, e: &PlanEdge) -> dot::Style {
        e.edge_style()
    }

    fn edge_color(&'a self, e: &PlanEdge) -> Option<dot::LabelText<'a>> {
        e.edge_color()
    }
}

impl<'a> dot::GraphWalk<'a, PlanNode, PlanEdge> for plan::With<'a, (Ident, DisplayConfig)> {
    fn nodes(&'a self) -> dot::Nodes<'a, PlanNode> {
        get_nodes(self.plan, &self.extended.1)
    }

    fn edges(&'a self) -> dot::Edges<'a, PlanEdge> {
        // TODO: use coroutines/generators to remove the need for multiple iters over operators and dataflow.
        //       - coroutines are currently on nightly, but not stable
        //       - libraries like [remit](https://docs.rs/remit/latest/remit/) can be used, but I want to
        //         reduce dependencies
        get_edges(self.plan, &self.extended.1)
    }

    fn source(&'a self, edge: &PlanEdge) -> PlanNode {
        edge.get_side(true)
    }

    fn target(&'a self, edge: &PlanEdge) -> PlanNode {
        edge.get_side(false)
    }
}