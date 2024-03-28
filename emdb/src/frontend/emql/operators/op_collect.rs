use super::*;

#[derive(Debug)]
pub struct Collect {
    call: Ident,
}

impl EMQLOperator for Collect {
    const NAME: &'static str = "collect";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            getident(),
            |call| Collect{call}
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
        let Self {call } = self;
        if let Some(Continue { data_type, prev_edge, last_span }) = cont {
            let out_edge = lp.operator_edges.insert(Edge::Null);

            let collect_op = lp.operators.insert(LogicalOperator { query: Some(qk), operator: LogicalOp::Collect { input: prev_edge, output: out_edge } });

            let call_span = call.span();
            let dt = Record {
                fields: HashMap::from([(call, RecordData::Scalar(ScalarType::Bag(data_type)))]),
                stream: false
            };
            lp.operator_edges[out_edge] = Edge::Uni { from: collect_op, with: dt.clone() };
            Ok(StreamContext::Continue(Continue { data_type: dt, prev_edge: out_edge, last_span: call_span }))

        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
