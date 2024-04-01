use crate::frontend::emql::parse::type_parser;

use super::*;

#[derive(Debug)]
pub struct Collect {
    call: Ident,
    field: Ident,
    datatype: AstType,
}

impl EMQLOperator for Collect {
    const NAME: &'static str = "collect";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, seqs!(
                getident(),
                matchident("as"),
                type_parser(isempty())
            )),
            |(call, (field, (_, datatype)))| Collect{call, field, datatype}
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
        let Self {call, field, datatype } = self;
        if let Some(Continue { data_type, prev_edge, last_span }) = cont {

            let out_edge = lp.operator_edges.insert(plan::DataFlow::Null);
            let collect_op = lp.operators.insert(
                plan::Operator { 
                    query: qk, 
                    kind: plan::OperatorKind::Pure(
                        plan::PureOperator::Collect { input: prev_edge, into: field.clone(), output: out_edge }
                    )
                }
            );

            let call_span = call.span();
            
            let bag_type = lp.scalar_types.insert(plan::ConcRef::Conc(plan::ScalarTypeConc::Bag(data_type.fields)));
            let record_type = lp.record_types.insert(plan::ConcRef::Conc(plan::RecordConc{ fields: HashMap::from([(field, bag_type)]) } )); 
            let out_type = plan::Data {
                fields: record_type,
                stream: false,
            };

            lp.operator_edges[out_edge] = plan::DataFlow::Incomplete { from: collect_op, with: out_type.clone()};
            
            Ok(StreamContext::Continue(Continue { data_type: out_type, prev_edge: out_edge, last_span: call_span }))

        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
