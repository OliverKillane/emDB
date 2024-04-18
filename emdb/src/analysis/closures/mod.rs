//! Generates closures for each [`plan::Context`] that can be used when building queries.
use crate::plan;
use proc_macro2::TokenStream;
use syn::{parse2, Type};
use quote::quote;

struct RustExpression {
    expression: TokenStream,
    data_type: Type,
}

impl RustExpression {
    fn from(expression: TokenStream, data_type: TokenStream) -> Self {
        Self {
            expression,
            data_type: parse2(data_type).expect("Invalid data type generated"),
        }
    }
}

fn translate_context(
    lp: &plan::Plan, 
    plan::Context { 
        ordering, 
        params, 
        returnflow, 
        discards 
    }: &plan::Context,
) -> RustExpression {

    let (trans_ops, trans_types): (Vec<_>, Vec<_>) = ordering.iter().map(|op_key| {
        let rs_expr = lp.get_operator(*op_key).generate_context_exprs(lp, *op_key, types);
        (rs_expr.expression, rs_expr.data_type)
    }).unzip();

    RustExpression::from(
        quote! {
            | #(#trans_params,)* | {
                (
                    #(#trans_ops,)*
                )
            }
        },
        quote!{ ( #(#trans_types,)* ) },
    )
}

#[enumtrait::store(generate_context_exprs_trait)]
trait GenerateContextExprs {
    fn generate_context_exprs(&self, lp: &plan::Plan, self_key: plan::Key<plan::Operator>) -> RustExpression;
}

#[enumtrait::impl_trait(generate_context_exprs_trait for plan::operator_enum)]
impl GenerateContextExprs for plan::Operator {}

impl GenerateContextExprs for plan::Update {}
impl GenerateContextExprs for plan::Insert {}
impl GenerateContextExprs for plan::Delete {}
impl GenerateContextExprs for plan::GetUnique {}
impl GenerateContextExprs for plan::ScanRefs {}
impl GenerateContextExprs for plan::DeRef {}
impl GenerateContextExprs for plan::Map {}
impl GenerateContextExprs for plan::Expand {}
impl GenerateContextExprs for plan::Fold {}
impl GenerateContextExprs for plan::Filter {}
impl GenerateContextExprs for plan::Sort {}
impl GenerateContextExprs for plan::Assert {}
impl GenerateContextExprs for plan::Collect {}
impl GenerateContextExprs for plan::Take {}
impl GenerateContextExprs for plan::Join {}
impl GenerateContextExprs for plan::GroupBy {}
impl GenerateContextExprs for plan::Fork {}
impl GenerateContextExprs for plan::Union {}
impl GenerateContextExprs for plan::Row {}
impl GenerateContextExprs for plan::Return {}
impl GenerateContextExprs for plan::Discard {}