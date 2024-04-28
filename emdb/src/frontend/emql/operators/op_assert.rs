//! Asserts an expression, aborting the query if it is false.

use super::*;


#[derive(Debug)]
pub struct Assert {
    call: Ident,
    expr: Expr,
}

impl EMQLOperator for Assert {
    const NAME: &'static str = "assert";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, setrepr(syn(collectuntil(isempty())), "<boolean expression>")),
            |(call, expr)| Assert { call, expr },
        )
    }

    fn build_logical(
        self,
        lp: &mut plan::Plan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        op_ctx: plan::Key<plan::Context>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, expr } = self;
        if let Some(cont) = cont {
            linear_builder(
                lp,
                op_ctx,
                cont,
                |lp, mo, prev, next_edge| {
                    Ok(
                        LinearBuilderState {
                            data_out: prev.data_type,
                            op:  plan::Assert { input: prev.prev_edge, assert: expr, output: next_edge }.into(),
                            call_span: call.span(),
                        }
                    )
                }
            )
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
