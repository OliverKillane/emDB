use std::iter::once;

use proc_macro2::{Span, TokenStream};
use quote_debug::Tokens;
use syn::{Expr, ExprClosure, Ident, Lifetime, Path, Type};

use crate::{
    backend::interface::namer::InterfaceNamer,
    plan::{self, RecordConc},
    utils::misc::{new_id, PushMap},
};
use quote::{quote, ToTokens};

const INTERNAL_FIELD_PREFIX: &str = "__internal_";

pub struct SerializedNamer {
    pub pulpit: pulpit::gen::namer::CodeNamer,
    pub struct_datastore: Ident,
    pub struct_database: Ident,
    pub mod_tables: Ident,
    pub db_lifetime: Tokens<Lifetime>,
    pub qy_lifetime: Tokens<Lifetime>,
    pub phantom_field: Ident,
    pub mod_queries: Ident,
    pub mod_queries_mod_query_enum_error: Ident,
    pub operator_error_parameter: Ident,
    pub interface: InterfaceNamer,
    pub struct_datastore_member_stats: Ident,
    pub struct_database_member_stats: Ident,
    pub struct_stats: Ident,
    pub closure_stats_param: Ident,
}

impl SerializedNamer {
    pub fn new() -> Self {
        let db_lifetime: Tokens<Lifetime> = quote!('db).into();
        Self {
            pulpit: pulpit::gen::namer::CodeNamer {
                lifetime_imm: db_lifetime.clone(),
                ..pulpit::gen::namer::CodeNamer::new(quote!(emdb::dependencies::pulpit).into())
            },
            struct_datastore: new_id("Datastore"),
            struct_database: new_id("Database"),
            mod_tables: new_id("tables"),
            db_lifetime,
            qy_lifetime: quote!('qy).into(),
            phantom_field: new_id(&format!("{INTERNAL_FIELD_PREFIX}phantomdata")),
            mod_queries: new_id("queries"),
            mod_queries_mod_query_enum_error: new_id("Error"),
            operator_error_parameter: new_id("err"),
            interface: InterfaceNamer::new(),
            struct_datastore_member_stats: new_id(&format!("{INTERNAL_FIELD_PREFIX}stats")),
            struct_database_member_stats: new_id(&format!("{INTERNAL_FIELD_PREFIX}stats")),
            struct_stats: new_id("Stats"),
            closure_stats_param: new_id(&format!("{INTERNAL_FIELD_PREFIX}stats")),
        }
    }

    pub fn transform_field_name(&self, name: &plan::RecordField) -> Ident {
        match name {
            plan::RecordField::User(i) => i.clone(),
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
        new_id(&format!("Record{}", key.arr_idx()))
            .into_token_stream()
            .into()
    }

    pub fn record_name_lifetimes(&self, key: plan::Key<plan::RecordType>) -> Tokens<Type> {
        self.lifetime_type_alias(new_id(&format!("Record{}", key.arr_idx())))
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

    pub fn operator_error_value_name(&self, key: plan::Key<plan::Operator>) -> Ident {
        new_id(&format!("Error{}", key.arr_idx()))
    }

    pub fn dataflow_value_name(&self, key: plan::Key<plan::DataFlow>) -> Ident {
        new_id(&format!("dataflow_value_{}", key.arr_idx()))
    }

    pub fn operator_error_variant_name(&self, key: plan::Key<plan::Operator>) -> Ident {
        new_id(&format!("Error{}", key.arr_idx()))
    }

    pub fn name_stat_member(&self, stats_ind: usize) -> Ident {
        Ident::new(&format!("stat_{stats_ind}"), Span::call_site())
    }

    pub fn access_stat_member(&self, stats_ind: usize) -> Tokens<Expr> {
        let Self {
            closure_stats_param,
            ..
        } = self;
        let member = self.name_stat_member(stats_ind);
        quote!( &#closure_stats_param.#member ).into()
    }

    pub fn table_internal_name(&self, lp: &plan::Plan, key: plan::Key<plan::Table>) -> Ident {
        // TODO: Separate internal names, currently we use the same name as the table, 
        //       as we expose the module. This is not ideal (potential for name 
        //       conflicts).
        lp.get_table(key).name.clone()
    }

    pub fn table_param_name(&self, lp: &plan::Plan, key: plan::Key<plan::Table>) -> Ident {
        new_id(&format!("internal_table_use_{}", lp.get_table(key).name))
    }
}

pub struct DataFlowNaming<'plan> {
    pub holding_var: Ident,
    pub stream: bool,
    pub data_constructor: Tokens<Type>,
    pub data_type: Tokens<Type>,
    pub record_type: &'plan plan::RecordConc,
}

/// Helper fn for getting the fields needed for accessing dataflow variables
pub fn dataflow_fields<'plan>(
    lp: &'plan plan::Plan,
    key: plan::Key<plan::DataFlow>,
    namer: &SerializedNamer,
) -> DataFlowNaming<'plan> {
    // NOTE: Previously we included the type of the stream/single, however this
    //       prevented using streams of types that are `impl Trait`.
    let SerializedNamer {
        db_lifetime,
        qy_lifetime,
        ..
    } = namer;
    let df_conn = lp.get_dataflow(key).get_conn();
    let record_index = lp.get_record_conc_index(df_conn.with.fields);
    let record_name = namer.record_name(*record_index);
    let record_type = match lp.get_record_type(*record_index) {
        plan::ConcRef::Conc(r) => r,
        plan::ConcRef::Ref(_) => unreachable!("Index is from lp.get_record_conc_index"),
    };

    DataFlowNaming {
        holding_var: namer.dataflow_value_name(key),
        stream: df_conn.with.stream,
        data_constructor: record_name.clone(),
        data_type: quote!(#record_name<#db_lifetime, #qy_lifetime>).into(),
        record_type,
    }
}

/// Helper fn for generating the construction for an error, and add it to the query's map of
/// error variants.
pub fn new_error(
    op_key: plan::Key<plan::Operator>,
    error_path: &Tokens<Path>,
    error_inner: Option<Tokens<Path>>,
    errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
    namer: &SerializedNamer,
) -> Tokens<Expr> {
    let variant_name = namer.operator_error_variant_name(op_key);
    let construct_error = if error_inner.is_some() {
        let param = &namer.operator_error_parameter;
        quote!(Err(#error_path::#variant_name(#param)))
    } else {
        quote!(Err(#error_path::#variant_name))
    }
    .into();
    errors.push(variant_name, error_inner);
    construct_error
}

/// Transfers fields of the same name, including a phantomdata member appended to the end.
pub fn transfer_fields<'brw>(
    from: &'brw Ident,
    record: &'brw RecordConc,
    namer: &'brw SerializedNamer,
) -> impl Iterator<Item = TokenStream> + 'brw {
    record
        .fields
        .keys()
        .map(move |id| {
            let field_name = namer.transform_field_name(id);
            quote!(#field_name: #from.#field_name)
        })
        .chain(once({
            let phantom_field = &namer.phantom_field;
            quote!(#phantom_field: std::marker::PhantomData)
        }))
}

pub fn expose_user_fields<'brw>(
    record: &'brw plan::RecordConc,
    namer: &'brw SerializedNamer,
) -> impl Iterator<Item = TokenStream> + 'brw {
    let phantomdata: &Ident = &namer.phantom_field;
    record
        .fields
        .keys()
        .map(|rf| {
            let field_name = namer.transform_field_name(rf);
            let alias = match rf {
                plan::RecordField::User(id) => quote! {#id},
                plan::RecordField::Internal(_) => quote! {_},
            };
            quote! {#field_name: #alias}
        })
        .chain(once(quote! {#phantomdata: _}))
}

pub fn boolean_predicate(
    lp: &plan::Plan,
    predicate: &Expr,
    dataflow: plan::Key<plan::DataFlow>,
    namer: &SerializedNamer,
) -> Tokens<ExprClosure> {
    let DataFlowNaming {
        data_constructor,
        data_type,
        record_type,
        ..
    } = dataflow_fields(lp, dataflow, namer);

    let input_fields = expose_user_fields(record_type, namer);

    quote! {
        |#data_constructor { #(#input_fields,)* } : &#data_type | -> bool {
            #predicate
        }
    }
    .into()
}
