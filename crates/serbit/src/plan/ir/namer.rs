use std::{hash::Hash};

pub trait Namer: std::fmt::Debug + Eq + 'static {
    /// A span, representing a location in the source
    type Span: From<Self::Ident> + Clone + std::fmt::Debug;

    /// A raw name (no span)
    type Name: Eq + Hash + From<Self::Ident> + Clone + std::fmt::Debug;

    /// An identifier, which includes a span
    type Ident: Eq + Hash + Clone + std::fmt::Debug;
}

#[derive(Debug)]
pub struct Assigned<N: Namer, Data> {
    pub ident: N::Ident,
    pub data: Data,
}

#[derive(Debug)]
pub struct Spanned<N: Namer, Data> {
    pub span: N::Span,
    pub data: Data,
}
