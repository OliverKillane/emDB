use crate::{
    groups::{Field, MutImmut},
    namer::CodeNamer,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ItemFn, ItemStruct, Type};

pub struct ImmConversion {
    pub imm_unpacked: Tokens<ItemStruct>,
    pub unpacker: Tokens<ItemFn>,
}

#[enumtrait::store(col_kind_trait)]
pub trait ColKind {
    fn derives(&self) -> MutImmut<Vec<Ident>>;
    fn generate_column_type(
        &self,
        namer: &CodeNamer,
        imm_type: Tokens<Type>,
        mut_type: Tokens<Type>,
    ) -> Tokens<Type> {
        let base_type = self.generate_base_type(namer);
        let generics = self.generate_generics(namer, imm_type, mut_type);
        quote! (#base_type #generics).into()
    }

    fn generate_base_type(&self, namer: &CodeNamer) -> Tokens<Type>;

    fn generate_generics(
        &self,
        _namer: &CodeNamer,
        imm_type: Tokens<Type>,
        mut_type: Tokens<Type>,
    ) -> TokenStream {
        quote! {<#imm_type, #mut_type>}
    }

    fn requires_get_lifetime(&self) -> bool {
        false
    }
    fn convert_imm(&self, namer: &CodeNamer, imm_fields: &[Field]) -> ImmConversion {
        let field_defs = imm_fields.iter().map(|Field { name, ty }| {
            quote! {
                pub #name : #ty
            }
        });
        let fields = imm_fields.iter().map(|Field { name, ty: _ }| name);
        let unpack_fields = fields.clone();

        let CodeNamer {
            mod_columns_struct_imm_unpacked,
            mod_columns_fn_imm_unpack,
            mod_columns_struct_imm,
            ..
        } = namer;

        ImmConversion {
            imm_unpacked: quote! {
                pub struct #mod_columns_struct_imm_unpacked {
                    #(#field_defs),*
                }
            }
            .into(),
            unpacker: quote! {
                pub fn #mod_columns_fn_imm_unpack(#mod_columns_struct_imm { #(#fields),* }: #mod_columns_struct_imm) -> #mod_columns_struct_imm_unpacked {
                    #mod_columns_struct_imm_unpacked { #(#unpack_fields),* }
                }
            }
            .into(),
        }
    }
    fn convert_imm_type(&self, field: &Field, _namer: &CodeNamer) -> Tokens<Type> {
        field.ty.clone()
    }
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
mod primary_pull;
pub use primary_pull::*;
mod primary_gen_arena;
pub use primary_gen_arena::*;
mod primary_thunderdome;
pub use primary_thunderdome::*;
mod primary_no_pull;
pub use primary_no_pull::*;
mod assoc_blocks;
pub use assoc_blocks::*;

impl AssocKind for AssocVec {}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(assoc_pull_enum)]
pub enum AssocPull {
    AssocVec,
}

impl AssocKind for AssocPull {}

#[enumtrait::impl_trait(col_kind_trait for assoc_pull_enum)]
impl ColKind for AssocPull {}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(assoc_app_enum)]
pub enum AssocApp {
    AssocVec,
    AssocBlocks,
}

impl AssocKind for AssocApp {}

#[enumtrait::impl_trait(col_kind_trait for assoc_app_enum)]
impl ColKind for AssocApp {}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pull_trans_enum)]
pub enum PullTrans {
    PrimaryPull,
    PrimaryRetain,
}

impl PrimaryKind for PullTrans {
    const TRANSACTIONS: bool = true;
    const DELETIONS: bool = true;

    type Assoc = AssocPull;
}

#[enumtrait::impl_trait(col_kind_trait for pull_trans_enum)]
impl ColKind for PullTrans {}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pull_enum)]
pub enum Pull {
    PrimaryGenArena,
    PrimaryThunderdome,
    PrimaryRetain,
    PrimaryPull,
}

#[enumtrait::impl_trait(col_kind_trait for pull_enum)]
impl ColKind for Pull {}

impl PrimaryKind for Pull {
    const TRANSACTIONS: bool = false;
    const DELETIONS: bool = true;

    type Assoc = AssocPull;
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(app_trans_enum)]
pub enum AppendTrans {
    PrimaryNoPull,
}

impl PrimaryKind for AppendTrans {
    const TRANSACTIONS: bool = true;
    const DELETIONS: bool = false;

    type Assoc = AssocApp;
}

#[enumtrait::impl_trait(col_kind_trait for app_trans_enum)]
impl ColKind for AppendTrans {}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(app_enum)]
pub enum Append {
    PrimaryNoPull,
}

impl PrimaryKind for Append {
    const TRANSACTIONS: bool = false;
    const DELETIONS: bool = false;

    type Assoc = AssocApp;
}

#[enumtrait::impl_trait(col_kind_trait for app_enum)]
impl ColKind for Append {}
