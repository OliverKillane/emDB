use proc_macro2::TokenStream;
use quote::quote;
use quote_debug::Tokens;
use std::collections::HashMap;
use syn::{ExprBlock, Field, Ident, Item, TraitItemFn, Type};

use crate::table::{HookStore, Namer, PushVec, Table};

pub struct FieldState {
    pub datatype: Tokens<Type>,
    pub init: Tokens<ExprBlock>,
}

#[enumtrait::store(access_gen_trait)]
pub trait AccessGen {
    fn gen(
        &self,
        data_field: usize,
        namer: &Namer,
        table: &Table,
        hooks: &mut HookStore,
        prelude: &mut PushVec<Tokens<Item>>,
        methods: &mut PushVec<Tokens<TraitItemFn>>,
    ) -> FieldState;
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(kind_enum)]
pub enum AccessKind {
    DebugAccess,
    Unique,
    Transactions,
}

#[enumtrait::impl_trait(access_gen_trait for kind_enum)]
impl AccessGen for AccessKind {}

pub struct DebugAccess;

impl AccessGen for DebugAccess {
    fn gen(
        &self,
        data_field: usize,
        namer: &Namer,
        table: &Table,
        hooks: &mut HookStore,
        prelude: &mut PushVec<Tokens<Item>>,
        methods: &mut PushVec<Tokens<TraitItemFn>>,
    ) -> FieldState {
        for (id, op) in &table.user_ops {
            hooks.push_hook(
                id,
                quote! { {println!("Before {:?}", #id);} }.into(),
                quote! { {println!("After {:?}", #id);} }.into(),
            );
        }

        FieldState {
            datatype: quote!(()).into(),
            init: quote!({ () }).into(),
        }
    }
}

pub struct Unique {
    field_id: Ident,
}

impl AccessGen for Unique {
    fn gen(
        &self,
        data_field: usize,
        namer: &Namer,
        table: &Table,
        hooks: &mut HookStore,
        prelude: &mut PushVec<Tokens<Item>>,
        methods: &mut PushVec<Tokens<TraitItemFn>>,
    ) -> FieldState {

        for (id, op) in &table.user_ops {
            // detect if op alters field
            // add hook to copy value
            // place in the field

            // detect if insert,
            // add key

            // detect if delete
            // add key
        }
        todo!()
    }
}

pub struct Transactions;

impl AccessGen for Transactions {
    fn gen(
        &self,
        data_field: usize,
        namer: &Namer,
        table: &Table,
        hooks: &mut HookStore,
        prelude: &mut PushVec<Tokens<Item>>,
        methods: &mut PushVec<Tokens<TraitItemFn>>,
    ) -> FieldState {

        // analyse all operations to get the log type
        // prelude with log type enum

        // add clone on pull hook and etc

        // add commit method

        todo!()
    }
}