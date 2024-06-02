use proc_macro2::Span;
use quote_debug::Tokens;
use syn::{Ident, Lifetime, Type};

use crate::plan;
use quote::{quote, ToTokens};

const INTERNAL_FIELD_PREFIX: &str = "__internal_";

pub struct SimpleNamer {
    pub pulpit: pulpit::gen::namer::CodeNamer,
    pub struct_datastore: Ident,
    pub struct_database: Ident,
    pub mod_tables: Ident,
    pub db_lifetime: Tokens<Lifetime>,
    pub qy_lifetime: Tokens<Lifetime>,
    pub phantom_field: Ident,
    pub mod_queries: Ident,
    pub mod_queries_mod_query_enum_error: Ident,
}

fn new_id(id: &str) -> Ident {
    Ident::new(id, Span::call_site())
}

impl SimpleNamer {
    pub fn new() -> Self {
        let db_lifetime: Tokens<Lifetime> = quote!('db).into();
        Self {
            pulpit: pulpit::gen::namer::CodeNamer{
                lifetime_imm: db_lifetime.clone(),
                ..pulpit::gen::namer::CodeNamer::new(quote!(emdb::dependencies::pulpit).into())
            },
            struct_datastore: new_id("Database"),
            struct_database: new_id("Window"),
            mod_tables: new_id("tables"),
            db_lifetime,
            qy_lifetime: quote!('qy).into(),
            phantom_field: new_id(&format!("{INTERNAL_FIELD_PREFIX}phantomdata")),
            mod_queries: new_id("queries"),
            mod_queries_mod_query_enum_error: new_id("Error"),
        }
    }

    pub fn transform_field_name(&self, name: &plan::RecordField) -> Ident {
        match name {
            plan::RecordField::User(i) => {
                i.clone()
            }
            plan::RecordField::Internal(i) => Ident::new(
                &format!("{}{}", INTERNAL_FIELD_PREFIX, i),
                Span::call_site(),
            ),
        }
    }

    pub fn pulpit_table_interaction(&self, key: plan::Key<plan::Operator>) -> Ident {
        new_id(&format!("pulpit_access_{}", key.arr_idx()))
    }

    pub fn record_name(&self, key: plan::Key<plan::RecordType>) -> Tokens<Type> {
        new_id(&format!("RecordTypeAlias{}", key.arr_idx()))
            .into_token_stream()
            .into()
    }

    
    pub fn record_name_lifetimes(&self, key: plan::Key<plan::RecordType>) -> Tokens<Type> {
        self.lifetime_type_alias(new_id(&format!("RecordTypeAlias{}", key.arr_idx())))
    }


    fn lifetime_type_alias(&self, id: Ident) -> Tokens<Type> {
        let Self {
            db_lifetime,
            qy_lifetime,
            ..
        } = self;
        quote! {
            #id<#db_lifetime, #qy_lifetime>
        }
        .into()
    }

    pub fn operator_closure_value_name(&self, key: plan::Key<plan::Operator>) -> Ident {
        new_id(&format!("operator_closure_value_{}", key.arr_idx()))
    }

    pub fn operator_return_value_name(&self, key: plan::Key<plan::Operator>) -> Ident {
        new_id(&format!("return_value_{}", key.arr_idx()))
    }

    pub fn operator_error_variant_name(&self, key: plan::Key<plan::Operator>) -> Ident {
        new_id(&format!("Error{}", key.arr_idx()))
    }

    pub fn dataflow_error(&self, key: plan::Key<plan::DataFlow>) -> Ident {
        new_id(&format!("dataflow_{}", key.arr_idx()))
    }

    
}
