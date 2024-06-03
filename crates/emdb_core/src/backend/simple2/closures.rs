//! generate the closures needed to use in the database, which capture parameters from the query.
//!

use std::collections::HashMap;

use crate::{
    plan,
    utils::misc::{PushMap, PushSet},
};
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprBlock, ExprClosure, ExprTuple, Ident, Path, Type};

use super::namer::SimpleNamer;
use super::operators::OperatorGen;
use super::tables::GeneratedInfo;
use super::types::generate_scalar_type;

pub fn unwrap_context(ctx: &plan::Context, namer: &SimpleNamer) -> Tokens<ExprTuple> {
    let values = ctx
        .ordering
        .iter()
        .map(|id| namer.operator_closure_value_name(*id));

    quote! {
        ( #(#values,)*)
    }
    .into()
}

pub fn generate_context_closure<'imm>(
    lp: &'imm plan::Plan,
    ctx: plan::Key<plan::Context>,
    get_types: &HashMap<plan::Idx<'imm, plan::Table>, HashMap<Ident, Tokens<Type>>>,
    namer: &SimpleNamer,
) -> Tokens<ExprClosure> {
    let context = lp.get_context(ctx);
    let data = generate_context(lp, context, get_types, namer);
    let args = context.params.iter().map(|(id, ty)| {
        let ty = generate_scalar_type(lp, get_types, *ty, namer);
        quote! { #id: #ty }
    });

    quote! {
        |#(#args),*| { #data }
    }
    .into()
}

pub fn generate_context<'imm>(
    lp: &'imm plan::Plan,
    ctx: &plan::Context,
    get_types: &HashMap<plan::Idx<'imm, plan::Table>, HashMap<Ident, Tokens<Type>>>,
    namer: &SimpleNamer,
) -> Tokens<ExprTuple> {
    let data = ctx
        .ordering
        .iter()
        .map(|op_key| lp.get_operator(*op_key).closure_data(lp, get_types, namer));
    quote! {
        (#(#data,)*)
    }
    .into()
}

/// Generate the code for a given context.
/// ```ignore
/// 'code_block: {
///    let op_1 = scan();
///    let op_2 = filter(op_1);
///    // ...
/// };
/// ```
///
pub fn generate_application<'imm>(
    lp: &'imm plan::Plan,
    ctx: &plan::Context,
    error_path: &Tokens<Path>,
    errors: &mut PushMap<Ident, Option<Tokens<Path>>>,
    mutated_tables: &mut PushSet<plan::ImmKey<'imm, plan::Table>>,
    gen_info: &GeneratedInfo<'imm>,
    namer: &SimpleNamer,
) -> Tokens<ExprBlock> {
    let num_errors_before = errors.len();

    let tokens = ctx
        .ordering
        .iter()
        .map(|op_key| {
            lp.get_operator(*op_key).apply(
                *op_key,
                lp,
                namer,
                error_path,
                errors,
                mutated_tables,
                gen_info,
            )
        })
        .collect::<Vec<_>>();
    let ret_val = if let Some(ret_op) = ctx.returnflow {
        let return_output = namer.operator_return_value_name(ret_op);
        quote! {#return_output}
    } else {
        quote! {()}
    };

    quote! {
        {
            #(#tokens;)*
            #ret_val
        }
    }
    .into()
}
