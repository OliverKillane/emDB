//! Asserts an expression, aborting the query if it is false.

use super::*;


#[derive(Debug)]
pub struct Assert {
    call: Ident,
    expr: Expr,
}

impl EMQLOperator for Assert {
    const NAME: &'static str = "assert";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, setrepr(syn(collectuntil(isempty())), "<boolean expression>")),
            |(call, expr)| Assert { call, expr },
        )
    }

    fn build_logical(
        self,
        lp: &mut plan::Plan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        mo: &mut Option<plan::Key<plan::Operator>>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, expr } = self;
        if let Some(cont) = cont {
            linear_builder(
                lp,
                qk,
                mo,
                cont,
                |lp, mo, prev, next_edge| {
                    Ok(
                        LinearBuilderState {
                            data_out: prev.data_type,
                            op_kind:  plan::OperatorKind::Pure(plan::PureOperator::Assert { input: prev.prev_edge, assert: expr, output: next_edge }),
                            call_span: call.span(),
                            update_mo: false,
                        }
                    )
                }
            )
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
