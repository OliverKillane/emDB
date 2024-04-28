use super::*;

#[derive(Debug)]
pub struct Return {
    call: Ident,
}

impl EMQLOperator for Return {
    const NAME: &'static str = "return";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(matchident(Self::NAME), |call| Return { call })
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
                let return_op = lp.operators.insert(plan::Return { input: prev_edge }.into());
                update_incomplete(lp.dataflow.get_mut(prev_edge).unwrap(), return_op);
                lp.get_mut_context(op_ctx).add_operator(return_op);
                // Node the return is set in super::super::sem, once it has checked for the duplicate returns
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
