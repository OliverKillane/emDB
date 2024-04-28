//! ## Context Closure Generation
//! Generate the closures required for a context, for use by operators.
//! ```
//! fn my_query(param1: i32) {
//!     // allow user's expressions to capture from query parameters
//!     let (op1_closure, op2_closure) = (
//!         | some_input: i32 | { some_input + param1 },
//!         | some_input: i32 | { some_input % 2 == 0 }
//!     );
//!
//!     // use generated closures in the operators
//!     // ...
//! }
//! ```

use super::names::ItemNamer;
use crate::plan;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Expr;

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
        Self {
            expression: quote! {()},
            datatype: quote! {()},
        }
    }
}

#[enumtrait::store(trans_operator_trait)]
trait OperatorClosures {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        None
    }
}

pub fn trans_context<Namer: ItemNamer>(
    lp: &plan::Plan,
    ctx_key: plan::Key<plan::Context>,
) -> ClosureArgs<'_> {
    let ctx = lp.get_context(ctx_key);

    let mut expressions = Vec::new();
    let mut data_types = Vec::new();
    for ClosureValue {
        expression,
        datatype,
    } in ctx
        .ordering
        .iter()
        .filter_map(|op_key| lp.get_operator(*op_key).gen_closure::<Namer>(*op_key, lp))
    {
        expressions.push(expression);
        data_types.push(datatype);
    }

    ClosureArgs {
        params: ctx
            .params
            .iter()
            .map(|(id, ty_idx)| (id, Namer::scalar_type(*ty_idx)))
            .collect(),
        value: ClosureValue {
            expression: quote! { ( #(#expressions ,)* ) },
            datatype: quote! { ( #(#data_types ,)* ) },
        },
    }
}

#[enumtrait::impl_trait(trans_operator_trait for plan::operator_enum)]
impl OperatorClosures for plan::Operator {}

impl OperatorClosures for plan::UniqueRef {}
impl OperatorClosures for plan::ScanRefs {}
impl OperatorClosures for plan::DeRef {}
impl OperatorClosures for plan::Insert {}
impl OperatorClosures for plan::Expand {}
impl OperatorClosures for plan::Delete {}
impl OperatorClosures for plan::Sort {}
impl OperatorClosures for plan::Collect {}
impl OperatorClosures for plan::Fork {}
impl OperatorClosures for plan::Union {}
impl OperatorClosures for plan::Return {}
impl OperatorClosures for plan::Discard {}

impl OperatorClosures for plan::Update {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        let (closure_expression, rec_out_ident) =
            mapping_expr::<Namer>(lp, self.update_type, self.mapping.iter());
        Some(single_expr::<Namer>(
            lp,
            self_key,
            self.input,
            closure_expression,
            rec_out_ident.into_token_stream(),
        ))
    }
}
impl OperatorClosures for plan::Fold {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        let (initial_values, rec_return) = mapto_dataflow::<Namer>(
            lp,
            self.output,
            self.fold_fields.iter().map(|(rf, ff)| (rf, &ff.initial)),
        );

        let (update_field, rec_return2) = mapto_dataflow::<Namer>(
            lp,
            self.output,
            self.fold_fields.iter().map(|(rf, ff)| (rf, &ff.update)),
        );
        let (update_using_previous, rec_return3) =
            dataflow_closure::<Namer>(lp, self.output, update_field);
        let (update_using_input, input_type) =
            dataflow_closure::<Namer>(lp, self.input, update_using_previous);

        assert_eq!(
            rec_return, rec_return2,
            "Return type of initial and update fields must be the same"
        );
        assert_eq!(
            rec_return, rec_return3,
            "Return type of initial and update fields must be the same"
        );

        Some(ClosureValue {
            expression: quote! {
                (#initial_values, #update_using_input)
            },
            datatype: quote! {
                (#rec_return, impl Fn(#input_type) -> (impl Fn(#rec_return) -> #rec_return))
            },
        })
    }
}
impl OperatorClosures for plan::Map {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        let (closure_expression, rec_out_ident) =
            mapto_dataflow::<Namer>(lp, self.output, self.mapping.iter().map(|(f, e)| (f, e)));
        Some(single_expr::<Namer>(
            lp,
            self_key,
            self.input,
            closure_expression,
            rec_out_ident.into_token_stream(),
        ))
    }
}
impl OperatorClosures for plan::Filter {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        Some(single_expr::<Namer>(
            lp,
            self_key,
            self.input,
            self.predicate.to_token_stream(),
            quote!(bool),
        ))
    }
}
impl OperatorClosures for plan::Assert {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        Some(single_expr::<Namer>(
            lp,
            self_key,
            self.input,
            self.assert.to_token_stream(),
            quote!(bool),
        ))
    }
}
impl OperatorClosures for plan::Take {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        Some(single_expr::<Namer>(
            lp,
            self_key,
            self.input,
            self.top_n.to_token_stream(),
            quote!(usize),
        ))
    }
}
impl OperatorClosures for plan::GroupBy {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        Some(context_namer::<Namer>(lp, self_key, self.inner_ctx))
    }
}
impl OperatorClosures for plan::ForEach {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        Some(context_namer::<Namer>(lp, self_key, self.inner_ctx))
    }
}
impl OperatorClosures for plan::Join {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        if let plan::MatchKind::Pred(pred) = &self.match_kind {
            let left_t = Namer::record_type(lp.get_dataflow(self.left).get_conn().with.fields);
            let right_t = Namer::record_type(lp.get_dataflow(self.right).get_conn().with.fields);
            Some(ClosureValue {
                expression: quote! {
                    move | left: &#left_t , right: &#right_t | {
                        let result: bool = #pred;
                        result
                    }
                },
                datatype: quote! {impl Fn(&#left_t, &#right_t) -> bool},
            })
        } else {
            None
        }
    }
}
impl OperatorClosures for plan::Row {
    fn gen_closure<Namer: ItemNamer>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &plan::Plan,
    ) -> Option<ClosureValue> {
        let (closure_expression, rec_out_ident) =
            mapto_dataflow::<Namer>(lp, self.output, self.fields.iter().map(|(f, e)| (f, e)));
        Some(ClosureValue {
            expression: closure_expression,
            datatype: rec_out_ident.into_token_stream(),
        })
    }
}

/// Given a dataflow as arguments, produce a closure that returns a type
fn single_expr<Namer: ItemNamer>(
    lp: &plan::Plan,
    op: plan::Key<plan::Operator>,
    df: plan::Key<plan::DataFlow>,
    expr: TokenStream,
    out_type: TokenStream,
) -> ClosureValue {
    let (closure, in_type) =
        dataflow_closure::<Namer>(lp, df, quote! { let result: #out_type = {#expr}; result });
    ClosureValue {
        expression: closure,
        datatype: quote! { Fn(&#in_type) -> #out_type },
    }
}

fn mapping_expr<'a, Namer: ItemNamer>(
    lp: &'a plan::Plan,
    output: plan::Key<plan::RecordType>,
    mapping: impl Iterator<Item = (&'a plan::RecordField, &'a Expr)>,
) -> (TokenStream, Ident) {
    let mut expressions = Vec::new();
    let mut fields = Vec::new();

    let data_types = &lp.get_record_type_conc(output).fields;
    let rec_out_ident = Namer::record_type(output);

    for (field, expr) in mapping {
        let expr_typename = Namer::scalar_type(data_types[field]);
        let field_name = Namer::record_field(field);
        expressions.push(quote! { let #field_name: #expr_typename = #expr ; });
        fields.push(field_name);
    }

    (
        quote! {
            { #(#expressions )*
            #rec_out_ident { #(#fields),* } }
        },
        rec_out_ident,
    )
}

/// Create a mapping to an output field from the data output required.
/// - `output` is the dataflow to output to, and thus the fields for the expressions to assign to.
fn mapto_dataflow<'a, Namer: ItemNamer>(
    lp: &'a plan::Plan,
    output: plan::Key<plan::DataFlow>,
    mapping: impl Iterator<Item = (&'a plan::RecordField, &'a Expr)>,
) -> (TokenStream, Ident) {
    let rec_out = lp.get_dataflow(output).get_conn().with.fields;
    mapping_expr::<Namer>(lp, rec_out, mapping)
}

/// Convert a context from an operator (e.g. [`plan::GroupBy`] or [`plan::ForEach`]) into a closure.
fn context_namer<Namer: ItemNamer>(
    lp: &plan::Plan,
    op: plan::Key<plan::Operator>,
    ctx: plan::Key<plan::Context>,
) -> ClosureValue {
    let ClosureArgs {
        params,
        value: ClosureValue {
            expression,
            datatype,
        },
    } = trans_context::<Namer>(lp, ctx);
    let params_tokens: Vec<_> = params.iter().map(|(id, ty)| quote! { #id : #ty }).collect();
    let inp_types: Vec<_> = params.iter().map(|(_, ty)| quote! { #ty }).collect();

    ClosureValue {
        expression: quote! { move | #(#params_tokens , )* | { #expression } },
        datatype: quote! { Fn( #(#inp_types ,)* ) -> ( #datatype ) },
    }
}

/// Generate a closure using a provided dataflow as input. Returns the identifier for the input type.
fn dataflow_closure<Namer: ItemNamer>(
    lp: &plan::Plan,
    df_in: plan::Key<plan::DataFlow>,
    inner: TokenStream,
) -> (TokenStream, Ident) {
    let record_key = lp.get_dataflow(df_in).get_conn().with.fields;
    let record_type = Namer::record_type(record_key);
    let params: Vec<_> = lp
        .get_record_type_conc(lp.get_dataflow(df_in).get_conn().with.fields)
        .fields
        .iter()
        .map(|(field_id, ty_idx)| {
            let id = Namer::record_field(field_id);
            quote! { #id }
        })
        .collect();

    (
        quote! {
            move | #record_type { #(#params ),* } | {
                #inner
            }
        },
        record_type,
    )
}
