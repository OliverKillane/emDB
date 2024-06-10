pub mod borrow;
pub mod count;
pub mod delete;
pub mod get;
pub mod insert;
pub mod scan;
pub mod transact;
pub mod unique_get;
pub mod update;

use quote_debug::Tokens;
use syn::{ItemImpl, ItemMod};

pub struct SingleOp {
    pub op_mod: Tokens<ItemMod>,
    pub op_impl: Tokens<ItemImpl>,
}

pub struct SingleOpFn {
    pub op_impl: Tokens<ItemImpl>,
}
