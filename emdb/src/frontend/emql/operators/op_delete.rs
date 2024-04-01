use self::plan::ScalarTypeConc;

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
        lp: &mut plan::LogicalPlan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        mo: &mut Option<plan::Key<plan::Operator>>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, field } = self;
        if let Some(Continue {
            data_type,
            prev_edge,
            last_span,
        }) = cont
        {
            if let Some(ts) = lp.get_record_type(data_type.fields).fields.get(&field) {
                if let ScalarTypeConc::TableRef(table_id) = lp.get_scalar_type(*ts) {
                    let table_id_copy = *table_id;
                    let out_edge = lp.operator_edges.insert(plan::DataFlow::Null);
                    let delete_op = lp.operators.insert(plan::Operator { query: qk, kind: plan::OperatorKind::Modify { modify_after: mo.clone(), op: plan::ModifyOperator::Delete { input: prev_edge, reference: field, table: table_id_copy, output: out_edge } } });
                    *mo = Some(delete_op.clone());
                    lp.operator_edges[out_edge] = plan::DataFlow::Incomplete {
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
