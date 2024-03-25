use proc_macro2::TokenStream;

use crate::plan::repr::LogicalPlan;
mod graphviz;
mod simple;

pub(crate) use graphviz::GraphViz;
pub(crate) use simple::Simple;
pub(crate) enum BackendTypes {
    Simple(Simple),
    GraphViz(GraphViz),
}

pub(crate) trait Backend {
    fn generate_code(plan: &LogicalPlan) -> TokenStream;
}
