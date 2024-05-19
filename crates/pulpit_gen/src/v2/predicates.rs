use quote_debug::Tokens;
use syn::{Ident, ItemFn};


pub struct Predicate {
    alias: Ident,
    tokens: Tokens<ItemFn>,
}
