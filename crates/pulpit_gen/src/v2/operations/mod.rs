pub mod borrow;
pub mod delete;
pub mod get;
pub mod insert;
pub mod transact;
pub mod update;

use quote_debug::Tokens;
use syn::{ItemImpl, ItemMod, ItemTrait};

pub struct SingleOp {
    pub op_mod: Tokens<ItemMod>,
    pub op_trait: Tokens<ItemTrait>,
    pub op_impl: Tokens<ItemImpl>,
}
