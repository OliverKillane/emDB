//! Take the top n items from the input stream.
use super::*;

#[derive(Debug)]
pub struct Take {
    call: Ident,
    expr: Expr
}

impl EMQLOperator for Take {
    const NAME: &'static str = "take";

    fn build_parser() -> impl TokenParser<Self> {
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
                |lp, mo, Continue { data_type, prev_edge, last_span }, next_edge| {
                    if data_type.stream {
                        Ok(
                            LinearBuilderState {
                                data_out: data_type,
                                op_kind: plan::OperatorKind::Pure(plan::Take { input: prev_edge, top_n: expr, output: next_edge }.into()),
                                call_span: call.span(),
                                update_mo: false
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