use std::collections::LinkedList;

use crate::{
    groups::{Field, MutImmut},
    namer::CodeNamer,
};
use proc_macro2::{Span, TokenStream};
use proc_macro_error2::{Diagnostic, Level};
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ItemFn, ItemStruct, Type};

pub struct ImmConversion {
    pub imm_unpacked: Tokens<ItemStruct>,
    pub unpacker: Tokens<ItemFn>,
}

// TODO: remove the strongly typed interface and replace with two enum types

#[enumtrait::store(col_kind_trait)]
pub trait ColKind {
    /// Required to check columns can be applied with the values provided.
    /// - Adapters take no values
    fn check_column_application(
        &self,
        error_span: Span,
        imm_fields: &[Field],
        mut_fields: &[Field],
        transactions: bool,
        deletions: bool,
    ) -> LinkedList<Diagnostic>;

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
                #[inline(always)]
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

mod primary_retain;
pub use primary_retain::*;
mod assoc_vec;
pub use assoc_vec::*;
mod primary_gen_arena;
pub use primary_gen_arena::*;
mod primary_thunderdome;
pub use primary_thunderdome::*;
mod assoc_blocks;
pub use assoc_blocks::*;
mod primary_thunderdome_trans;
pub use primary_thunderdome_trans::*;
mod assoc_app_vec;
pub use assoc_app_vec::*;
mod assoc_pull_block;
pub use assoc_pull_block::*;

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(enum_primary)]
pub enum Primary {
    AssocBlocks,
    PrimaryRetain,
    PrimaryThunderdome,
    PrimaryThunderDomeTrans,
    PrimaryGenArena,
    AssocAppVec,
}

#[enumtrait::impl_trait(col_kind_trait for enum_primary)]
impl ColKind for Primary {}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(enum_associated)]
pub enum Associated {
    AssocVec,
    AssocBlocks,
    AssocAppVec,
    AssocPullBlocks,
}

#[enumtrait::impl_trait(col_kind_trait for enum_associated)]
impl ColKind for Associated {}
