use std::any::type_name;

use crate::{
    index::IndexGen,
    table::{PushVec, Table},
};
use proc_macro2::TokenStream;
use quote_debug::Tokens;
use syn::{ExprBlock, Ident, Item, TraitItemFn};

use quote::{quote, ToTokens};
#[enumtrait::quick_enum]
#[enumtrait::store(ops_kind_enum)]
pub enum OperationKind {
    Insert,
    Update,
    Get,
    Brw,
    BrwMut,
    Count,
}

#[enumtrait::store(ops_gen_trait)]
pub trait OpGen {
    fn generate(
        &self,
        name: &Ident,
        table: &Table,
        prelude: &mut PushVec<Tokens<Item>>,
        tks_before: &[Tokens<ExprBlock>],
        tks_after: &[Tokens<ExprBlock>],
    ) -> Tokens<TraitItemFn> {
        let type_name = std::any::type_name::<Self>();
        quote! {
            /// TODO: this operation is unimplemented for
            fn #name(&self) {
                let this_op: #type_name;
                #(#tks_before)*
                let do_op = ();
                #(#tks_after)*
            }
        }
        .into()
    }
}

#[enumtrait::impl_trait(ops_gen_trait for ops_kind_enum)]
impl OpGen for OperationKind {}

pub struct Insert;

impl OpGen for Insert {}

pub struct Update;

impl OpGen for Update {}

pub struct Get;

impl OpGen for Get {}

pub struct Brw;

impl OpGen for Brw {}

pub struct BrwMut;

impl OpGen for BrwMut {}

pub struct Count;

impl OpGen for Count {}
