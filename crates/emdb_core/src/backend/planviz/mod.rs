//! # Logical Plan Vizualisation
//! The debugging plan graph view for emDB.
//!
//! Given the complexity of the [`plan::Plan`], [`crate::analysis`] and
//! [`crate::optimise`] it is necessary to explore plans graphically.
//!
//! ## Live Debugging
//! It is recommended to work in a scratch file, with Planviz implemented.
//! - If using vscode, the [graphviz interactive preview extension](vscode:extension/tintinweb.graphviz-interactive-preview)
//!   is recommended (open dots file, save in scratch rust file and watch preview
//!   automatically update live).

use std::{fs::File, path::Path};

use super::{plan, EMDBBackend, Ident, LinkedList, TokenStream};
use crate::utils::{misc::singlelist, on_off::on_off};
use combi::{
    core::setrepr,
    tokens::{
        basic::{collectuntil, isempty, syn},
        options::{OptEnd, OptField, OptParse},
        TokenDiagnostic, TokenIter,
    },
    Combi, Repr,
};
use dot;
use proc_macro_error2::Diagnostic;
use quote::quote;
use syn::LitStr;
use typed_generational_arena::StandardArena as GenArena;

mod edges;
mod errors;
mod nodes;

use edges::{get_edges, EdgeStyle, PlanEdge};
use nodes::{get_nodes, node_call, PlanNode, StyleableNode};

pub struct PlanViz {
    out_location: LitStr,
    config: DisplayConfig,
}

struct DisplayConfig {
    display_types: bool,
    display_ctx_ops: bool,
    display_control: bool,
}

impl EMDBBackend for PlanViz {
    const NAME: &'static str = "PlanViz";

    fn parse_options(
        backend_name: &Ident,
        options: Option<TokenStream>,
    ) -> Result<Self, LinkedList<Diagnostic>> {
        let parser = setrepr(
            (
                OptField::new("path", || {
                    setrepr(syn(collectuntil(isempty())), "<file path>")
                }),
                (
                    OptField::new("types", on_off),
                    (
                        OptField::new("ctx", on_off),
                        (OptField::new("control", on_off), OptEnd),
                    ),
                ),
            )
                .gen('='),
            "path = <path>, types = <on|off>, ctx = <on|off>, control = <on|off>",
        );
        if let Some(opts) = options {
            let (_, res) = parser.comp(TokenIter::from(opts, backend_name.span()));
            let (path, (types, (ctx, (control, _)))) =
                res.to_result().map_err(TokenDiagnostic::into_list)?;

            if let Some(out_location) = path {
                Ok(PlanViz {
                    out_location,
                    config: DisplayConfig {
                        display_types: types.unwrap_or(false),
                        display_ctx_ops: ctx.unwrap_or(false),
                        display_control: control.unwrap_or(false),
                    },
                })
            } else {
                Err(singlelist(errors::expected_path(backend_name)))
            }
        } else {
            Err(singlelist(errors::expected_options(
                backend_name,
                &format!("{}", Repr(&parser)),
            )))
        }
    }

    fn generate_code(
        self,
        impl_name: Ident,
        plan: &plan::Plan,
    ) -> Result<TokenStream, LinkedList<Diagnostic>> {
        let out_path_str = self.out_location.value();
        match File::create(Path::new(&out_path_str)) {
            Ok(mut open_file) => {
                match dot::render(
                    &plan::With {
                        plan,
                        extended: (impl_name.clone(), self.config),
                    },
                    &mut open_file,
                ) {
                    Ok(()) => Ok(quote! {
                    mod #impl_name {
                        pub const OUT_DIRECTORY: &str = #out_path_str;
                    }
                                        }),
                    Err(e) => Err(singlelist(errors::io_error(
                        &impl_name,
                        self.out_location.span(),
                        &e,
                    ))),
                }
            }
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
