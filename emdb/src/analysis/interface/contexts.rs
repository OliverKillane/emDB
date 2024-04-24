//! Generate a context for each type, and an id.
//! 
//! 
//! 
//! 
//! 

use crate::plan::{self, RecordConc};
use proc_macro2::{Ident, TokenStream, Span};
use syn::{parse2, Expr, Pat, Type};
use quote::{quote, ToTokens};

use super::names::ItemNamer;

fn op_pattern(key: plan::Key<plan::Operator>) -> Ident {
    Ident::new(&format!("capture_op_{}", key.to_idx()), Span::call_site())
}

pub struct ClosureArgs<'a> {
    pub params: Vec<(&'a Ident, Ident)>,
    pub value: ClosureValue,
}

pub struct ClosureValue {
    pub expression: TokenStream,
    pub datatype: TokenStream,
}

impl ClosureValue {
    fn empty() -> Self {
        Self { expression: quote!{()}, datatype: quote!{()} }
    }

    fn todo() -> Self {
        Self { expression: quote!{todo!()}, datatype: quote! { TODO } }
    }
}

#[enumtrait::store(trans_operator_trait)]
trait OperatorClosures {
    fn gen_closure<Namer: ItemNamer>(&self, self_key: plan::Key<plan::Operator>, lp: &plan::Plan) -> ClosureValue;
}

pub fn trans_context<'a, Namer: ItemNamer>(lp: &'a plan::Plan, ctx_key: plan::Key<plan::Context>) -> ClosureArgs<'a> { 
    let ctx = lp.get_context(ctx_key);

    let mut expressions = Vec::new();
    let mut data_types = Vec::new();
    for ClosureValue { expression, datatype } in ctx.ordering.iter().map(|op_key| lp.get_operator(*op_key).gen_closure::<Namer>(*op_key, lp)) {
        expressions.push(expression);
        data_types.push(datatype);
    }

    ClosureArgs {
        params: ctx.params.iter().map(|(id, ty_idx)| (id, Namer::scalar_type(*ty_idx) )).collect(),
        value: ClosureValue {
            expression: quote!{ ( #(#expressions ,)* ) },
            datatype: quote!{ ( #(#data_types ,)* ) },
    }}
}

fn dataflow_closure<Namer: ItemNamer>(lp: &plan::Plan, df_in: plan::Key<plan::DataFlow>, inner: TokenStream) -> (TokenStream, Ident) {
    let record_key = lp.get_dataflow(df_in).get_conn().with.fields;
    let record_type = Namer::record_type(record_key);
    let params: Vec<_> = lp.get_record_type_conc(lp.get_dataflow(df_in).get_conn().with.fields).fields.iter().map(|(field_id, ty_idx)| {
        let id =  Namer::record_field(field_id);
        quote! { #id }
    }).collect();

    (quote! {
        | #record_type { #(#params ),* } | {
            #inner
        }
    }, record_type)
}

#[enumtrait::impl_trait(trans_operator_trait for plan::operator_enum)]
impl OperatorClosures for plan::Operator {}

impl OperatorClosures for plan::UniqueRef {
    fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }
}
impl OperatorClosures for plan::ScanRefs {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}
impl OperatorClosures for plan::DeRef {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}
impl OperatorClosures for plan::Insert {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}
impl OperatorClosures for plan::Expand {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}
impl OperatorClosures for plan::Delete {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}
impl OperatorClosures for plan::Sort {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}
impl OperatorClosures for plan::Collect {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}
impl OperatorClosures for plan::Fork {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}
impl OperatorClosures for plan::Union {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}
impl OperatorClosures for plan::Return {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}
impl OperatorClosures for plan::Discard {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::empty()
    }}


impl OperatorClosures for plan::Update {
    fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::todo() // TODO: Change
    }
}
impl OperatorClosures for plan::Fold {
    fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::todo() // TODO: Change
    }
}
impl OperatorClosures for plan::Map {
    fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        let mut expressions = Vec::new();
        let mut fields = Vec::new();
        
        let rec_out = lp.get_dataflow(self.output).get_conn().with.fields;
        let data_types = &lp.get_record_type_conc(rec_out).fields;
        let rec_out_ident = Namer::record_type(rec_out);

        for (field, expr) in &self.mapping {
            let expr_typename = Namer::scalar_type(data_types[field]);
            let field_name = Namer::record_field(field);
            expressions.push(quote!{ let #field_name: #expr_typename = #expr ; });
            fields.push(field_name);
        }

        let closure_expression = quote!{
            #(#expressions )*
            #rec_out_ident { #(#fields),* }
        };

        single_expr::<Namer>(lp, self_key, self.input, closure_expression, rec_out_ident.into_token_stream())
    }
}
impl OperatorClosures for plan::Filter {
    fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        single_expr::<Namer>(lp, self_key, self.input, self.predicate.clone().into_token_stream(), quote!(bool))
    }
}
impl OperatorClosures for plan::Assert {
    fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        single_expr::<Namer>(lp, self_key, self.input, self.assert.clone().into_token_stream(), quote!(bool))
    }
}
impl OperatorClosures for plan::Take {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::todo() // TODO: Change
    }}
impl OperatorClosures for plan::GroupBy {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::todo() // TODO: Change
    }}
impl OperatorClosures for plan::ForEach {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::todo() // TODO: Change
    }}
impl OperatorClosures for plan::Join {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::todo() // TODO: Change
    }}
impl OperatorClosures for plan::Row {fn gen_closure<Namer:ItemNamer>(&self,self_key:plan::Key<plan::Operator>,lp: &plan::Plan) -> ClosureValue {
        ClosureValue::todo() // TODO: Change
    }}

fn single_expr<Namer: ItemNamer>(lp: &plan::Plan, op: plan::Key<plan::Operator>, df: plan::Key<plan::DataFlow>, expr: TokenStream, out_type: TokenStream) -> ClosureValue {
    let (closure, in_type) = dataflow_closure::<Namer>(lp, df, quote!{ let result: #out_type = {#expr}; result });
    ClosureValue { 
        expression: closure, 
        datatype: quote!{ Fn(&#in_type) -> #out_type } 
    }
}

fn context_namer<Namer: ItemNamer>(lp: &plan::Plan, op: plan::Key<plan::Operator>, ctx: plan::Key<plan::Context>) -> ClosureValue {
    let ClosureArgs { params, value: ClosureValue { expression, datatype } } = trans_context::<Namer>(lp, ctx);
    let params_tokens: Vec<_> = params.iter().map(|(id, ty)| quote! { #id : #ty }).collect();
    let inp_types: Vec<_> = params.iter().map(|(_, ty)| quote! { #ty }).collect();
    
    ClosureValue {
        expression: quote!{ | #(#params_tokens , )* | { #expression } },
        datatype: quote!{ Fn( #(#inp_types ,)* ) -> ( #datatype ) }
    }
}

