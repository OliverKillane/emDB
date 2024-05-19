//! Borrow immutably from the pulpit.

use quote_debug::Tokens;
use syn::{ItemImpl, ItemStruct, ItemTrait};
use quote::quote;

use super::columns::{Groups, PrimaryKind};

struct BorrowData {
    borrow_struct: Tokens<ItemStruct>,
    borrow_trait: Tokens<ItemTrait>,
    borrow_impl: Tokens<ItemImpl>,
}

fn generate_borrow<Primary: PrimaryKind>(groups: &Groups<Primary>) -> BorrowData {
    // struct for borrow
    // trait borrow
    // impl borrow for Window<'imm> {}

    BorrowData {
        borrow_struct: quote!{

        }.into(),
        borrow_trait: quote!{

        }.into(),
        borrow_impl: quote!{
            
        }.into(),
    }
}
