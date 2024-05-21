use std::any::type_name;

use crate::{
    namer::Namer,
    table::{PushVec, Table},
};
use proc_macro2::TokenStream;
use quote_debug::Tokens;
use syn::{ExprBlock, Ident, Item, TraitItemFn, Type};

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
    ) -> Tokens<TraitItemFn>;
}

#[enumtrait::impl_trait(ops_gen_trait for ops_kind_enum)]
impl OpGen for OperationKind {}

pub struct Insert {
    input_struct: Tokens<Type>,
}

impl OpGen for Insert {
    fn generate(
        &self,
        name: &Ident,
        table: &Table,
        namer: &Namer,
        prelude: &mut PushVec<Tokens<Item>>,
    ) -> Tokens<TraitItemFn> {
        todo!()
    }
}

pub struct Update;

impl OpGen for Update {
    fn generate(
        &self,
        name: &Ident,
        table: &Table,
        namer: &Namer,
        prelude: &mut PushVec<Tokens<Item>>,
    ) -> Tokens<TraitItemFn> {
        todo!()
    }
}

pub struct Get;

impl OpGen for Get {
    fn generate(
        &self,
        name: &Ident,
        table: &Table,
        namer: &Namer,
        prelude: &mut PushVec<Tokens<Item>>,
    ) -> Tokens<TraitItemFn> {
        todo!()
    }
}

pub struct Delete;

impl OpGen for Delete {
    fn generate(
        &self,
        name: &Ident,
        table: &Table,
        namer: &Namer,
        prelude: &mut PushVec<Tokens<Item>>,
    ) -> Tokens<TraitItemFn> {
        todo!()
    }
}

pub struct Count;

impl OpGen for Count {
    fn generate(
        &self,
        name: &Ident,
        table: &Table,
        namer: &Namer,
        prelude: &mut PushVec<Tokens<Item>>,
    ) -> Tokens<TraitItemFn> {
        todo!()
    }
}

// Before Hooks: () -> Result<A, E>, After Hooks (A) -> Result<(), E>
