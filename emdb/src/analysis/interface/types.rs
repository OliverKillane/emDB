//! ## Type Definition Generation
//! Each type is given an alias based on its name using the [`ItemNamer`] trait.
//! The definitions of these aliases are produced by the [`TypeImplementor`] trait.
//!
//! [`SimpleTypeImplementor`] is a basic implementation.
use std::{collections::HashSet, marker::PhantomData};

use super::names::ItemNamer;
use crate::plan;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

/// A conveinent abstraction to apply transformations to types, and uses `type`
/// aliases to define references to types.
pub trait TypeImplementor {
    type Namer: ItemNamer;

    fn translate_scalar(
        &self,
        name: Ident,
        key: plan::Key<plan::ScalarType>,
        scalar: &plan::ScalarTypeConc,
    ) -> TokenStream;

    fn translate_record(
        &self,
        name: Ident,
        key: plan::Key<plan::RecordType>,
        record: &plan::RecordConc,
    ) -> TokenStream;

    fn translate_table_ref(&self, table: plan::Key<plan::Table>) -> TokenStream;

    fn trans_scalar_type(
        &self,
        ty_key: plan::Key<plan::ScalarType>,
        ty: &plan::ScalarType,
    ) -> TokenStream {
        let self_ty = Self::Namer::scalar_type(ty_key);
        match ty {
            plan::ConcRef::Conc(scalar) => self.translate_scalar(self_ty, ty_key, scalar),
            plan::ConcRef::Ref(ref_ty) => {
                let ref_name = Self::Namer::scalar_type(*ref_ty);
                quote! {
                    type #self_ty = #ref_name;
                }
            }
        }
    }

    fn trans_record_type(
        &self,
        ty_key: plan::Key<plan::RecordType>,
        ty: &plan::RecordType,
    ) -> TokenStream {
        let self_ty = Self::Namer::record_type(ty_key);
        match ty {
            plan::ConcRef::Conc(record) => self.translate_record(self_ty, ty_key, record),
            plan::ConcRef::Ref(ref_ty) => {
                let ref_name = Self::Namer::record_type(*ref_ty);
                quote! {
                    type #self_ty = #ref_name;
                }
            }
        }
    }

    fn translate_all_types(&self, lp: &plan::Plan) -> TokenStream {
        let mut tks = TokenStream::new();
        tks.extend(
            lp.record_types
                .iter()
                .map(|(key, ty)| self.trans_record_type(key, ty)),
        );
        tks.extend(
            lp.scalar_types
                .iter()
                .map(|(key, ty)| self.trans_scalar_type(key, ty)),
        );
        tks.extend(
            lp.tables
                .iter()
                .map(|(key, _)| self.translate_table_ref(key)),
        );
        tks
    }
}

/// We discard the generation number when tracking indexes from an immutable plan
type KeyIdx = usize;

/// A basic [`TypeImplementor`], generates public types for parameters and
/// returns.
pub struct SimpleTypeImplementor<Namer: ItemNamer> {
    namer: PhantomData<Namer>,
    public_records: HashSet<KeyIdx>,
}

fn add_public_record(
    lp: &plan::Plan,
    set: &mut HashSet<KeyIdx>,
    location: plan::Key<plan::RecordType>,
) {
    match lp.get_record_type(location) {
        plan::ConcRef::Conc(rec) => {
            set.insert(location.to_idx());
            for scalar in rec.fields.values() {
                add_public_scalar(lp, set, *scalar)
            }
        }
        plan::ConcRef::Ref(inner) => add_public_record(lp, set, *inner),
    }
}

fn add_public_scalar(
    lp: &plan::Plan,
    set: &mut HashSet<KeyIdx>,
    location: plan::Key<plan::ScalarType>,
) {
    match lp.get_scalar_type(location) {
        plan::ConcRef::Conc(scalar) => {
            if let plan::ScalarTypeConc::Record(r) = scalar {
                add_public_record(lp, set, *r)
            }
        }
        plan::ConcRef::Ref(inner) => add_public_scalar(lp, set, *inner),
    }
}

impl<Namer: ItemNamer> SimpleTypeImplementor<Namer> {
    pub fn with_public_types(lp: &plan::Plan) -> Self {
        let mut public_records = HashSet::new();

        for (_, query) in &lp.queries {
            let context = lp.get_context(query.ctx);
            if let Some(ret_op) = context.returnflow {
                let rec_idx = lp
                    .get_dataflow(lp.get_operator(ret_op).get_return().input)
                    .get_conn()
                    .with
                    .fields;
                add_public_record(lp, &mut public_records, rec_idx);
            }
            for (_, param) in &context.params {
                add_public_scalar(lp, &mut public_records, *param);
            }
        }

        Self {
            namer: PhantomData,
            public_records,
        }
    }
}

impl<Namer: ItemNamer> TypeImplementor for SimpleTypeImplementor<Namer> {
    type Namer = Namer;

    fn translate_scalar(
        &self,
        name: Ident,
        key: plan::Key<plan::ScalarType>,
        scalar: &plan::ScalarTypeConc,
    ) -> TokenStream {
        let set_to = match scalar {
            plan::ScalarTypeConc::TableRef(t) => Self::Namer::table_ref(*t).to_token_stream(),
            plan::ScalarTypeConc::Bag(b) => quote! {()},
            plan::ScalarTypeConc::Record(r) => Self::Namer::record_type(*r).to_token_stream(),
            plan::ScalarTypeConc::Rust(ty) => ty.to_token_stream(),
        };
        quote! {
            type #name = #set_to;
        }
    }

    fn translate_record(
        &self,
        name: Ident,
        key: plan::Key<plan::RecordType>,
        record: &plan::RecordConc,
    ) -> TokenStream {
        let vis = if self.public_records.contains(&key.to_idx()) {
            quote!(pub)
        } else {
            quote!()
        };

        let fields: Vec<_> = record
            .fields
            .iter()
            .map(|(rf, ty)| {
                let rf_id = Self::Namer::record_field(rf);
                let ty_id = Self::Namer::scalar_type(*ty);
                let rf_pre = if let plan::RecordField::User(_) = rf {
                    quote!(#vis #rf_id)
                } else {
                    rf_id.into_token_stream()
                };
                quote! { #rf_pre: #ty_id }
            })
            .collect();

        quote! {
            #vis struct #name {
                #(#fields , )*
            }
        }
    }

    fn translate_table_ref(&self, table: plan::Key<plan::Table>) -> TokenStream {
        let table_name = Self::Namer::table_ref(table);
        quote! {
            /// Reference to the table
            pub struct #table_name {}
        }
    }
}
