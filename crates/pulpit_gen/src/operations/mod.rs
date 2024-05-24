pub mod borrow;
pub mod delete;
pub mod get;
pub mod insert;
pub mod transact;
pub mod update;

use quote_debug::Tokens;
use syn::{ItemImpl, ItemMod};

pub struct SingleOp {
    pub op_mod: Tokens<ItemMod>,
    pub op_impl: Tokens<ItemImpl>,
}
