use crate::{
    groups::{Field, MutImmut},
    namer::CodeNamer,
};
use proc_macro2::Span;
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ItemFn, ItemStruct, Lifetime, Type};

pub struct ImmConversion {
    pub imm_unpacked: Tokens<ItemStruct>,
    pub unpacker: Tokens<ItemFn>,
}

pub trait ColKind {
    fn derives(&self) -> MutImmut<Vec<Ident>>;
    fn generate_column_type(
        &self,
        namer: &CodeNamer,
        imm_type: Tokens<Type>,
        mut_type: Tokens<Type>,
    ) -> Tokens<Type>;

    fn generate_column_type_no_generics(&self, namer: &CodeNamer) -> Tokens<Type>;
    fn requires_get_lifetime(&self) -> bool;
    fn convert_imm(&self, namer: &CodeNamer, imm_fields: &[Field]) -> ImmConversion;
    fn convert_imm_type(&self, field: &Field, namer: &CodeNamer) -> Tokens<Type>;
}

pub trait AssocKind: ColKind {}

pub trait PrimaryKind: ColKind {
    const TRANSACTIONS: bool;
    const DELETIONS: bool;
    type Assoc: AssocKind;
}

mod primary_retain;
pub use primary_retain::*;
mod assoc_vec;
pub use assoc_vec::*;

// TODO: remove (for testing)
impl AssocKind for AssocVec {}

impl PrimaryKind for PrimaryRetain {
    const TRANSACTIONS: bool = true;
    const DELETIONS: bool = true;
    type Assoc = AssocVec;
}
