use super::*;

#[derive(Debug)]
pub struct Ref {
    call: Ident,
    table_name: Ident,
    out_as: Ident,
}

impl EMQLOperator for Ref {
    const NAME: &'static str = "ref";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(seqs!(
            matchident("ref"), 
            getident(),
            matchident("as"),
            getident()
        ), |(call, (table_name, (_, out_ref)))| {
            Ref { call, table_name, out_as: out_ref }
        })
    }

    fn build_logical(
        self,
        lp: &mut plan::Plan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        mo: &mut Option<plan::Key<plan::Operator>>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, table_name, out_as: out_ref } = self;
        if cont.is_none() {
            if let Some(table_id) = tn.get(&table_name) {
                let access = plan::TableAccess::Ref(out_ref.clone());
                let record_out = plan::Data {
                    fields: generate_access(*table_id, access.clone(), lp, None).unwrap(),
                    stream: true
                };
                let out_edge = lp.dataflow.insert(plan::DataFlow::Null);
                let ref_op = lp.operators.insert(plan::Operator {
                    query: qk,
                    kind: plan::OperatorKind::Access { access_after: *mo, op: plan::AccessOperator::Scan { access , table: *table_id, output: out_edge } },
                });
                
                *lp.get_mut_dataflow(out_edge) = plan::DataFlow::Incomplete {
                    from: ref_op,
                    with: record_out.clone(),
                };

                Ok(StreamContext::Continue(Continue {
                    data_type: record_out,
                    prev_edge: out_edge,
                    last_span: call.span(),
                }))
            } else {
                Err(singlelist(errors::query_nonexistent_table(
                    &call,
                    &table_name,
                )))
            }
        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
