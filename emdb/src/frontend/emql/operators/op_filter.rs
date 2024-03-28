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
        lp: &mut LogicalPlan,
        tn: &HashMap<Ident, TableKey>,
        qk: QueryKey,
        vs: &mut HashMap<Ident, VarState>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, filter_expr } = self;
        if let Some(prev) = cont {
            let out_edge = lp.operator_edges.insert(Edge::Null);
            let assert_op = lp.operators.insert(LogicalOperator { query: Some(qk), operator: LogicalOp::Filter { input: prev.prev_edge, predicate: filter_expr, output: out_edge } });
            lp.operator_edges[out_edge] = Edge::Uni { from: assert_op, with: prev.data_type.clone() };

            Ok(StreamContext::Continue(Continue { data_type: prev.data_type, prev_edge: out_edge, last_span: call.span() }))
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
