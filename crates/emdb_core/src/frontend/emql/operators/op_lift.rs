use super::*;

#[derive(Debug)]
pub struct Lift {
    call: Ident,
    contents: Vec<StreamExpr>
}

impl EMQLOperator for Lift {
    const NAME: &'static str = "lift";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(functional_style(Self::NAME, 
            ctx_recur
        ),|(call, contents)| Lift {
            call,
            contents,
        })
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
        let Self { call, contents } = self;
        if let Some(Continue { data_type, prev_edge, last_span }) = cont {
            let mut errors = LinkedList::new();
            let next_edge = lp.dataflow.insert(plan::DataFlow::Null);
            let inner_ctx = lp.contexts.insert(plan::Context::from_params(lp.get_record_type_conc(data_type.fields).fields.iter().filter_map(|(field, ty)| {
                // NOTE: Here we disallow the use of internal fields in a lift.
                //       - We lift to provide values to the user's closures, as 
                //         internals cannot be used in user's closures, there is 
                //         no point in lifting.
                match field {
                    plan::RecordField::User(i) => Some((i.clone(), *ty)),
                    plan::RecordField::Internal(_) => None,
                }
            }).collect()));

            let foreach_op = lp.operators.insert(plan::Lift { 
                input: prev_edge,
                inner_ctx, 
                output: next_edge 
            }.into());

            update_incomplete(lp.get_mut_dataflow(prev_edge), foreach_op);

            let mut variables = HashMap::new();

            add_streams_to_context(lp, tn, ts, &mut variables, inner_ctx, contents, &call, &mut errors);
            discard_ends(lp, inner_ctx, variables);

            lp.get_mut_context(op_ctx).add_operator(foreach_op);

            if let Some(out_stream) = lp.get_context(inner_ctx).returnflow {
                if errors.is_empty() {
                    if let plan::Operator::Return(plan::Return{ input }) = lp.get_operator(out_stream) {
                        let old_data = lp.get_dataflow(*input).get_conn().with.clone();
                        assert!(!old_data.stream, "return always takes single");
                        let new_data = plan::Data { fields: old_data.fields, stream: data_type.stream };
                        *lp.get_mut_dataflow(next_edge) = plan::DataFlow::Incomplete { from: foreach_op, with: new_data.clone() };

                        Ok( StreamContext::Continue(Continue { data_type: new_data, prev_edge: next_edge, last_span: call.span() }))
                    
                    } else {
                        unreachable!("By invariant of the return, it can only be a return operator");
                    }
                } else {
                    Err(errors)
                }
            } else {
                errors.push_back(errors::no_return_in_context(&call));
                Err(errors)
            }
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}