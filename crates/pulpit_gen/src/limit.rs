use quote_debug::Tokens;
use syn::{Expr, Ident};
use quote::quote;

pub enum LimitKind {
    /// Used when the limit provided is known to the gen macro
    /// TODO: Perform optimisations the size of the indices
    Literal(usize),

    /// Used to provide generic-level information for pulpit columns to use.
    /// - e.g. A const index for the size of the column
    /// TODO: Implement pulpit table that is a single buffer
    ConstVal(Tokens<Expr>),
}

pub struct Limit {
    pub value: LimitKind,
    pub alias: Ident,
}

impl Limit {
    pub fn generate_check(&self) -> Tokens<Expr> {
        match &self.value {
            LimitKind::Literal(l) => quote!{#l},
            LimitKind::ConstVal(expr) => quote!{
                {
                    const VALUE: usize = { (#expr) as usize };
                    VALUE
                }
            },
        }.into()
    }
}