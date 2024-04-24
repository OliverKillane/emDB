//! TODO:
//! Generate aliases for all
//! Take some struct that generates the implementations for each type

use crate::plan;
use super::names::ItemNamer;

use proc_macro2::{TokenStream, Ident};
use quote::quote;

pub fn translate_all_types(lp: &plan::Plan, ty_impl: &impl TypeImplementor) -> TokenStream {
    let mut tks = TokenStream::new();
    tks.extend(lp.record_types.iter().map(|(key, ty)| ty_impl.trans_record_type(key, ty)));
    tks.extend(lp.scalar_types.iter().map(|(key, ty)| ty_impl.trans_scalar_type(key, ty)));
    tks.extend(lp.tables.iter().map(|(key, _)| ty_impl.translate_table_ref(key)));
    tks
}

/// A conveinent abstraction to apply transformations to types, and uses `type` 
/// aliases to define references to types.
pub trait TypeImplementor {
    type Namer: ItemNamer;

    fn translate_scalar(&self, name: Ident, scalar: &plan::ScalarTypeConc) -> TokenStream;
    fn translate_record(&self, name: Ident, record: &plan::RecordConc) -> TokenStream;
    fn translate_table_ref(&self, table: plan::Key<plan::Table>) -> TokenStream;
    
    fn trans_scalar_type(&self, ty_key: plan::Key<plan::ScalarType>, ty: &plan::ScalarType) -> TokenStream {
        let self_ty = Self::Namer::scalar_type(ty_key);
        match ty {
            plan::ConcRef::Conc(scalar) => self.translate_scalar(self_ty, scalar),
            plan::ConcRef::Ref(ref_ty) => {
                let ref_name = Self::Namer::scalar_type(*ref_ty);
                quote! {
                    type #self_ty = #ref_name;
                }
            },
        }
    }

    fn trans_record_type(&self, ty_key: plan::Key<plan::RecordType>, ty: &plan::RecordType) -> TokenStream {
        let self_ty = Self::Namer::record_type(ty_key);
        match ty {
            plan::ConcRef::Conc(record) => self.translate_record(self_ty, record),
            plan::ConcRef::Ref(ref_ty) => {
                let ref_name = Self::Namer::record_type(*ref_ty);
                quote! {
                    type #self_ty = #ref_name;
                }
            },
        }
    }
}
