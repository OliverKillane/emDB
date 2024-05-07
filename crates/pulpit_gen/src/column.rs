use super::FieldType;
use proc_macro2::TokenStream;
use quote::quote;

#[enumtrait::store(column_gen_trait)]
pub trait Gen {
    fn generate(&self, mut_fields: &[FieldType], imm_fields: &[FieldType]) -> TokenStream;
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(column_kind_enum)]
pub enum Kind {
    VecCol,
}

#[enumtrait::impl_trait(column_gen_trait for column_kind_enum)]
impl Gen for Kind {}

pub struct VecCol;

impl Gen for VecCol {
    fn generate(&self, mut_fields: &[FieldType], imm_fields: &[FieldType]) -> TokenStream {
        quote! {
            VecCol<(#(#mut_fields),*), (#(#imm_fields),*)>
        }
    }
}
