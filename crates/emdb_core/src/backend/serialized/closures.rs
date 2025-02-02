//! generate the closures needed to use in the database, which capture parameters from the query.

use crate::{
    plan,
    utils::{
        misc::PushMap,
        mut_scope::{Mutability, ScopeHandle},
    },
};
use itertools::Itertools;
use quote::{quote, ToTokens};
use quote_debug::Tokens;
use syn::{ExprClosure, ExprTuple, Ident, Path};

use super::operators::OperatorGen;
use super::tables::GeneratedInfo;
use super::types::generate_scalar_type;
use super::{
    namer::{dataflow_fields, DataFlowNaming, SerializedNamer},
    operators::OperatorImpl,
    stats::RequiredStats,
};

pub struct ContextGen<'parent_scope, 'imm> {
    pub code: Tokens<ExprClosure>,
    pub can_error: bool,
    pub scope: ScopeHandle<'parent_scope, plan::ImmKey<'imm, plan::Table>>,
}

/// Generate the code for a given context.
/// - Includes a parameter for aliasing `self` (rather than the closure
///   borrowing `self`)
#[allow(clippy::too_many_arguments)]
pub fn generate_application<'imm, 'brw, 'scope: 'brw>(
    lp: &'imm plan::Plan,
    ctx: plan::Key<plan::Context>,
    error_path: &Tokens<Path>,
    errors: &mut PushMap<'_, Ident, Option<Tokens<Path>>>,
    parent_scope: &'brw mut ScopeHandle<'scope, plan::ImmKey<'imm, plan::Table>>,
    gen_info: &GeneratedInfo<'imm>,
    namer: &SerializedNamer,
    operator_impl: &OperatorImpl,
    required_stats: &mut RequiredStats,
) -> ContextGen<'brw, 'imm> {
    let context = lp.get_context(ctx);
    let SerializedNamer {
        mod_tables,
        db_lifetime,
        pulpit:
            pulpit::gen::namer::CodeNamer {
                struct_window,
                ..
            },
        closure_stats_param,
        struct_stats,
        ..
    } = namer;
    let mut context_vals = Vec::new();

    let error_cnt = errors.count();

    let mut scope = parent_scope.scope();
    let scope_ref = &mut scope;

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
                scope_ref,
                gen_info,
                &mut context_vals,
                operator_impl,
                required_stats,
            )
        })
        .collect::<Vec<_>>();

    let (ids, vals): (Vec<_>, Vec<_>) = context_vals.into_iter().unzip();

    let can_error = errors.count() > error_cnt;

    let tables = scope
        .mutabilities()
        .sorted_by_key(|(k, _)| k.arr_idx())
        .map(|(t, mutable)| {
            let mod_name = namer.table_internal_name(lp, **t);
            let ty = quote!(#mod_tables::#mod_name::#struct_window<#db_lifetime>);
            let usage_name = namer.table_param_name(lp, **t);
            let reference = match mutable {
                Mutability::Mut => quote!(&mut),
                Mutability::Imm => quote!(&),
            };
            quote!(#usage_name: #reference #ty)
        });

    let stats = quote!(#closure_stats_param: &#struct_stats);

    let params = context.params.iter().map(|(id, ty)| {
        let ty = generate_scalar_type(lp, &gen_info.get_types, *ty, namer);
        quote! { #id: #ty }
    });

    let inflows = context.inflows.iter().map(|df| {
        let DataFlowNaming { holding_var, .. } = dataflow_fields(lp, *df, namer);
        quote!(#holding_var)
    });

    let ret_val = if let Some(ret_op) = context.returnflow {
        let return_output = namer.operator_return_value_name(ret_op);
        if can_error {
            quote!(Ok(#return_output))
        } else {
            quote!(#return_output)
        }
    } else if can_error {
        quote!(Ok(()))
    } else {
        quote!()
    };

    ContextGen {
        code: quote! {
            |#stats, #(#tables,)* #(#params,)* #(#inflows,)* | {
                let ( #(#ids),* ) = ( #(#vals),* );
                #(#tokens;)*
                #ret_val
            }
        }
        .into(),
        can_error,
        scope,
    }
}

pub fn generate_closure_usage<'imm>(
    lp: &'imm plan::Plan,
    namer: &SerializedNamer,
    params: impl Iterator<Item=impl ToTokens>,
    inflows: impl Iterator<Item=impl ToTokens>,
    scope: &ScopeHandle<'_, plan::ImmKey<'imm, plan::Table>>,
) -> Tokens<ExprTuple> {
    let SerializedNamer { closure_stats_param, .. } = namer;
    let tables = scope.mutabilities().map(|(k, _)| {
        namer.table_param_name(lp, **k)
    });
    quote!{
        (#closure_stats_param, #(#tables,)* #(#params,)* #(#inflows,)*)
    }.into()
}
