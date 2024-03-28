use super::*;

#[derive(Debug)]
pub struct Return {
    call: Ident,
}

impl EMQLOperator for Return {
    const NAME: &'static str = "return";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(matchident(Self::NAME), |call| Return { call })
    }

    fn build_logical(
        self,
        lp: &mut LogicalPlan,
        tn: &HashMap<Ident, TableKey>,
        qk: QueryKey,
        vs: &mut HashMap<Ident, VarState>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call } = self;
        if let Some(Continue {
            data_type,
            prev_edge,
            last_span,
        }) = cont
        {
            if data_type.stream {
                Err(singlelist(errors::query_cannot_return_stream(last_span, call.span())))
            } else {
                let return_op = lp.operators.insert(LogicalOperator {
                    query: Some(qk),
                    operator: LogicalOp::Return { input: prev_edge },
                });
    
                Ok(StreamContext::Returned(ReturnVal {
                    span: call.span(),
                    index: return_op,
                }))
            }
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
