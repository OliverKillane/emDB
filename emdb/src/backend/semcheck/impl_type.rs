use crate::{analysis::interface::{names::{DefaultNamer, ItemNamer}, types}, plan::{self, RecordField}};
use proc_macro2::{TokenStream, Ident};
use quote::{quote, ToTokens};

pub struct SemCheckTypes;

impl types::TypeImplementor for SemCheckTypes {
    type Namer = DefaultNamer;

    fn translate_scalar(&self, name: Ident, scalar: &plan::ScalarTypeConc) -> TokenStream {
        let set_to = match scalar {
            plan::ScalarTypeConc::TableRef(t) => Self::Namer::table(*t).to_token_stream(),
            plan::ScalarTypeConc::Bag(b) => quote!{()},
            plan::ScalarTypeConc::Record(r) => Self::Namer::record_type(*r).to_token_stream(),
            plan::ScalarTypeConc::Rust(ty) => ty.to_token_stream(),
        };
        quote! {
            type #name = #set_to;
        }
    }

    fn translate_record(&self, name: Ident, record: &plan::RecordConc) -> TokenStream {
        let fields: Vec<_> = record.fields.iter().map(|(rf, ty)| {
            let rf_id = Self::Namer::record_field(rf);
            let ty_id = Self::Namer::scalar_type(*ty);
            quote! { #rf_id: #ty_id }
        }).collect();
        quote! {
            struct #name {
                #(#fields , )*
            }
        }
    }
    
    fn translate_table_ref(&self, table: plan::Key<plan::Table>) -> TokenStream {
        let table_name = Self::Namer::table(table);
        quote!{
            /// Reference to the table
            struct #table_name {}
        }
    }
}