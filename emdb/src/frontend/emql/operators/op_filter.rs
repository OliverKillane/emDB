use self::plan::DataFlow;

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
            functional_style(Self::NAME, syn(collectuntil(isempty()))),
            |(call, filter_expr)| Filter { call, filter_expr },
        )
    }

    fn build_logical(
        self,
        lp: &mut plan::LogicalPlan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        mo: &mut Option<plan::Key<plan::Operator>>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, filter_expr } = self;
        if let Some(prev) = cont {
            let out_edge = lp.operator_edges.insert(plan::DataFlow::Null);
            let assert_op = lp.operators.insert(plan::Operator { query: qk, kind: plan::OperatorKind::Pure(plan::PureOperator::Filter { input: prev.prev_edge, predicate: filter_expr, output: out_edge}) });

            lp.operator_edges[out_edge] = DataFlow::Incomplete { from: assert_op, with: prev.data_type.clone() };

            Ok(StreamContext::Continue(Continue { data_type: prev.data_type, prev_edge: out_edge, last_span: call.span() }))
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
