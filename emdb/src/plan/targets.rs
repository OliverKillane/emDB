use proc_macro2::Ident;
use std::collections::HashMap;

pub(crate) enum Target {
    Simple,
    Graphviz,
}

pub(crate) struct Targets {
    pub(crate) backends: HashMap<Ident, Target>,
}
