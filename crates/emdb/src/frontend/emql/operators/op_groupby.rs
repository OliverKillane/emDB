use super::*;

#[derive(Debug)]
pub struct GroupBy {
    call: Ident,
    by: Ident, 
    in_name: Ident,
    contents: Vec<StreamExpr>
}

impl EMQLOperator for GroupBy {
    const NAME: &'static str = "groupby";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(functional_style(Self::NAME, 
            seqs!(
                setrepr(getident(), "<field to group by>"),
                matchident("for"),
                matchident("let"),
                setrepr(getident(), "<variable to use in context>"),
                matchident("in"),
                recovgroup(Delimiter::Brace, setrepr(ctx_recur, "<context>"))
            )
        ),|(call, (by, (_, (_, (in_name, (_, contents))))))| GroupBy {
            call,
            by,
            in_name,
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
        let Self { call, by, in_name, contents } = self;
        if let Some(Continue { data_type, prev_edge, last_span }) = cont {
            let mut errors = LinkedList::new();

            // check in is stream
            if !data_type.stream {
                errors.push_back(errors::operator_requires_streams2(&call));
            }

            let by_rec_field = by.clone().into();

            // check field exists
            let mut in_fields = lp.get_record_type_conc(data_type.fields).clone();
            if let Some(grouping_type) = in_fields.fields.remove(&by_rec_field) {
                let inner_rec = lp.record_types.insert(plan::ConcRef::Conc(in_fields));
                let next_edge = lp.dataflow.insert(plan::DataFlow::Null);
                let stream_in_edge = lp.dataflow.insert(plan::DataFlow::Null);
                let inner_ctx = lp.contexts.insert(plan::Context::from_params(vec![(by, grouping_type)]));

                     
                let groupby_op = lp.operators.insert(plan::GroupBy {
                    input: prev_edge, 
                    group_by: by_rec_field, 
                    stream_in: stream_in_edge, 
                    inner_ctx, 
                    output: next_edge 
                }.into());

                update_incomplete(lp.get_mut_dataflow(prev_edge), groupby_op);

                let stream_in_data = plan::Data { fields: inner_rec, stream: true }; // NOTE: true for groupby but false for foreach
                *lp.get_mut_dataflow(stream_in_edge) = plan::DataFlow::Incomplete { from: groupby_op, with: stream_in_data.clone() };

                let inner_cont: Continue = Continue {
                    data_type: stream_in_data,
                    prev_edge: stream_in_edge,
                    last_span: call.span(),
                };

                let mut variables = HashMap::from([
                    (in_name.clone(), VarState::Available { created: in_name.span(), state: inner_cont })
                ]);
                
                add_streams_to_context(lp, tn, ts, &mut variables, inner_ctx, contents, &call, &mut errors);
                discard_ends(lp, inner_ctx, variables);
                lp.get_mut_context(op_ctx).add_operator(groupby_op);
                
                if let Some(out_stream) = lp.get_context(inner_ctx).returnflow {
                    if errors.is_empty() {
                        if let plan::Operator::Return(plan::Return{ input }) = lp.get_operator(out_stream) {
                            let old_data = lp.get_dataflow(*input).get_conn().with.clone();
                            assert!(!old_data.stream, "return always takes single");
                            let new_data = plan::Data { fields: old_data.fields, stream: true };
                            *lp.get_mut_dataflow(next_edge) = plan::DataFlow::Incomplete { from: groupby_op, with: new_data.clone() };
    
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
                errors.push_back(
                    errors::access_field_missing(&call, &by, 
                        get_user_fields(&in_fields)
                            
                        )
                    );
                Err(errors)
            }
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}