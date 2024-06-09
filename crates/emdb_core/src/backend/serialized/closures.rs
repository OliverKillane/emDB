//! generate the closures needed to use in the database, which capture parameters from the query.
//!


use crate::{
    plan,
    utils::misc::{PushMap, PushSet},
};
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprClosure, Ident, Path};

use super::namer::{dataflow_fields, DataFlowNaming, SerializedNamer};
use super::operators::OperatorGen;
use super::tables::GeneratedInfo;
use super::types::generate_scalar_type;

pub struct ContextGen {
    pub code: Tokens<ExprClosure>,
    pub can_error: bool,
    pub mutates: bool,
}

/// Generate the code for a given context.
/// - Includes a parameter for aliasing `self` (rather than the closure
///   borrowing `self`)
pub fn generate_application<'imm, 'brw>(
    lp: &'imm plan::Plan,
    ctx: plan::Key<plan::Context>,
    error_path: &Tokens<Path>,
    errors: &mut PushMap<'brw, Ident, Option<Tokens<Path>>>,
    mutated_tables: &mut PushSet<'brw, plan::ImmKey<'imm, plan::Table>>,
    gen_info: &GeneratedInfo<'imm>,
    namer: &SerializedNamer,
) -> ContextGen {
    let context = lp.get_context(ctx);
    let SerializedNamer { self_alias, .. } = namer;
    let mut context_vals = Vec::new();

    let error_cnt = errors.count();
    let mut_cnt = mutated_tables.count();

    let tokens = context
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
                &mut context_vals,
            )
        })
        .collect::<Vec<_>>();

    let (ids, vals): (Vec<_>, Vec<_>) = context_vals.into_iter().unzip();

    let can_error = errors.count() > error_cnt;
    let mutates = mutated_tables.count() > mut_cnt;
    

    let params = context.params.iter().map(|(id, ty)| {
        let ty = generate_scalar_type(lp, &gen_info.get_types, *ty, namer);
        quote! { #id: #ty }
    });

    let inflows = context.inflows.iter().map(|df| {
        let DataFlowNaming { holding_var, dataflow_type, .. } = dataflow_fields(lp, *df, namer);
        quote!(#holding_var: #dataflow_type)
    });

    let ret_val = if let Some(ret_op) = context.returnflow {
        let return_output = namer.operator_return_value_name(ret_op);
        if can_error {
            quote!(Ok(#return_output))
        } else {
            quote!(#return_output)
        }
    } else if can_error {
        quote! (Ok(()))
    } else {
        quote!()
    };

    let self_mut = if mutates {
        quote! {mut}
    } else {
        quote! {}
    };

    ContextGen {
        code: quote! {
            |#self_alias: & #self_mut Self , #(#params,)* #(#inflows,)* | {
                let ( #(#ids),* ) = ( #(#vals),* );
                #(#tokens;)*
                #ret_val
            }
        }
        .into(),
        can_error,
        mutates,
    }
}
