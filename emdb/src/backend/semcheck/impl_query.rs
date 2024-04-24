use proc_macro2::TokenStream;
use crate::{analysis::interface::{contexts::{trans_context, ClosureArgs}, names::{DefaultNamer, ItemNamer}, query}, plan};
use quote::quote;

pub fn translate_all_queries(lp: &plan::Plan) -> TokenStream {
    lp.queries.iter().map(|(key, query)| translate_query(lp, key, query)).collect()
}

fn translate_query(lp: &plan::Plan, qk: plan::Key<plan::Query>, query: &plan::Query) -> TokenStream {
    let ClosureArgs { params, value } = trans_context::<DefaultNamer>(lp, query.ctx);

    let query_params = params.iter().map(|(id, ty)| {
        quote! { #id: #ty }
    });
    let query_name = &query.name;
    let query_closure_gen = value.expression;
    let query_closure_type = value.datatype;

    let return_type = if let Some(ret_op) = lp.get_context(query.ctx).returnflow {
        let ret = lp.get_operator(ret_op).get_return();
        let ret_type = DefaultNamer::record_type(lp.get_dataflow(ret.input).get_conn().with.fields);
        quote! { -> #ret_type }
    } else {
        quote!()
    };

    quote!{
        pub fn #query_name(#(#query_params ,)*) #return_type {
            let closures = #query_closure_gen ;
            
            todo!()
        }
    }
}