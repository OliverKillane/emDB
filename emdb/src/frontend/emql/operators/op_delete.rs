use super::*;

#[derive(Debug)]
pub struct Delete {
    call: Ident,
    field: Ident,
}

impl EMQLOperator for Delete {
    const NAME: &'static str = "delete";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(functional_style(Self::NAME, getident()), |(call, field)| {
            Delete { call, field }
        })
    }

    fn build_logical(
        self,
        lp: &mut LogicalPlan,
        tn: &HashMap<Ident, TableKey>,
        qk: QueryKey,
        vs: &mut HashMap<Ident, VarState>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, field } = self;
        if let Some(Continue {
            data_type,
            prev_edge,
            last_span,
        }) = cont
        {
            if let Some(ts) = data_type.fields.get(&field) {
                if let RecordData::Scalar(ScalarType::Ref(table_id)) = ts {
                    let out_edge = lp.operator_edges.insert(Edge::Null);
                    let delete_op = lp.operators.insert(LogicalOperator {
                        query: Some(qk),
                        operator: LogicalOp::Delete {
                            input: prev_edge,
                            reference: field,
                            table: *table_id,
                            output: out_edge,
                        },
                    });
                    lp.operator_edges[out_edge] = Edge::Uni {
                        from: delete_op,
                        with: data_type.clone(),
                    };

                    Ok(StreamContext::Continue(Continue {
                        data_type,
                        prev_edge: out_edge,
                        last_span: call.span(),
                    }))
                } else {
                    Err(singlelist(errors::query_delete_field_not_reference(
                        lp, &call, &field, ts,
                    )))
                }
            } else {
                Err(singlelist(errors::query_delete_field_not_present(
                    &call, &field,
                )))
            }
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
