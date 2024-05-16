use std::any::type_name;

use crate::{
    table::{Namer, PushVec, Table},
};
use proc_macro2::TokenStream;
use quote_debug::Tokens;
use syn::{ExprBlock, Ident, Item, TraitItemFn};

use quote::{quote, ToTokens};
#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(ops_kind_enum)]
pub enum OperationKind {
    Insert,
    Update,
    Get,
    Delete,
    Count,
}

#[enumtrait::store(ops_gen_trait)]
pub trait OpGen {
    fn generate(
        &self,
        name: &Ident,
        table: &Table,
        namer: &Namer,
        prelude: &mut PushVec<Tokens<Item>>,
        tks_before: &[Tokens<ExprBlock>],
        tks_after: &[Tokens<ExprBlock>],
    ) -> Tokens<TraitItemFn> {
        let type_name = std::any::type_name::<Self>();
        quote! {
            /// TODO: this operation is unimplemented for
            fn #name(&self) -> () {
                let this_op: &str = #type_name;
                {#(#tks_before)*}
                let do_op = ();
                {#(#tks_after)*}
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

pub struct Delete;

impl OpGen for Delete {}

pub struct Count;

impl OpGen for Count {
    fn generate(
        &self,
        name: &Ident,
        table: &Table,
        namer: &Namer,
        prelude: &mut PushVec<Tokens<Item>>,
        tks_before: &[Tokens<ExprBlock>],
        tks_after: &[Tokens<ExprBlock>],
    ) -> Tokens<TraitItemFn> {
        let prim_name = namer.primary_column();
        quote! {
            fn #name(&self) -> usize {
                #(#tks_before)*
                self.#prim_name.len()
                #(#tks_after)*
            }
        }.into()
    }
}
