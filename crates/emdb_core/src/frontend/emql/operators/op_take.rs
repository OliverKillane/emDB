//! Take the top n items from the input stream.
use super::*;

#[derive(Debug)]
pub struct Take {
    call: Ident,
    expr: Expr
}

impl EMQLOperator for Take {
    const NAME: &'static str = "take";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                setrepr(syn(collectuntil(isempty())), "<expression for the number to take>"),
            ),
            |(call, expr)| Take { call, expr },
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
                |lp, mo, Continue { data_type, prev_edge, last_span }, next_edge| {
                    if data_type.stream {
                        Ok(
                            LinearBuilderState {
                                data_out: data_type,
                                op: (plan::Take { input: prev_edge, limit: expr, output: next_edge }.into()),
                                call_span: call.span()
                            }
                        )
                    } else {
                        Err(singlelist(errors::query_stream_single_connection(call.span(), last_span, true)))
                    }
                }
            )
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}