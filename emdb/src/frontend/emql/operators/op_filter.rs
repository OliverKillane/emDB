//! Apply a predicate to filter rows
use super::*;

#[derive(Debug)]
pub struct Filter {
    call: Ident,
    filter_expr: Expr,
}

impl EMQLOperator for Filter {
    const NAME: &'static str = "filter";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, setrepr(syn(collectuntil(isempty())), "<filter predicate>")),
            |(call, filter_expr)| Filter { call, filter_expr },
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
        let Self { call, filter_expr } = self;
        if let Some(prev) = cont {
            linear_builder(
                lp,
                qk,
                mo,
                prev,
                |lp, mo, prev, next_edge| {
                    Ok(
                        LinearBuilderState {
                            data_out: prev.data_type,
                            op_kind: plan::OperatorKind::Pure(plan::Filter { input: prev.prev_edge, predicate: filter_expr, output: next_edge}.into()),
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
