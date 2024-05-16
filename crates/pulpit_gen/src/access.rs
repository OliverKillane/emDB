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
        for (id, op) in &table.operations {
            hooks.push_hook(
                id,
                quote! { {println!("Before {:?}", #id);} }.into(),
                quote! { {println!("After {:?}", #id);} }.into(),
            );
        }

        FieldState {
            datatype: quote!(()).into(),
            init: quote!({()}).into(),
        }
    }
}
