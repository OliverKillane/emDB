use core::prelude;
use std::collections::HashMap;

use crate::access::{AccessGen, AccessKind, FieldState};
use crate::ops::{OpGen, OperationKind};
use crate::table::{PushVec, Table};
use bimap::BiHashMap;
use proc_macro2::TokenStream;
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprBlock, Ident, Item, TraitItemFn, Type};

struct GroupID {
    idx: usize,
}
struct SubcolID {
    mutable: bool,
    place: usize,
}
struct FieldID {
    grp: GroupID,
    sub: SubcolID,
}

pub struct Field {
    name: Ident,
    ty: Tokens<Type>,
}

pub struct Group {
    pub col: ColumnKind,
    pub mut_fields: Vec<Field>,
    pub imm_fields: Vec<Field>,
}

pub struct Columns {
    // TODO: Decide structure
    d: Vec<Group>,
}

impl Columns {
    pub fn groups(&self) -> impl Iterator<Item = (usize, &Group)> {
        self.d.iter().enumerate()
    }
}

#[enumtrait::quick_enum]
#[enumtrait::store(column_kind_enum)]
pub enum ColumnKind {
    Basic,
}

#[enumtrait::store(column_gen_trait)]
pub trait ColumnGen {
    fn gen_state(&self, id: &Ident, mut_fields: &[Field], imm_fields: &[Field]) -> FieldState;
}

#[enumtrait::impl_trait(column_gen_trait for column_kind_enum)]
impl ColumnGen for ColumnKind {}

pub struct Basic;

impl ColumnGen for Basic {
    fn gen_state(&self, id: &Ident, mut_fields: &[Field], imm_fields: &[Field]) -> FieldState {
        todo!()
    }
}
