use super::*;

#[derive(Debug)]
pub struct Ref {
    call: Ident,
    table_name: Ident,
}

impl EMQLOperator for Ref {
    const NAME: &'static str = "ref";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(seq(matchident("ref"), getident()), |(call, table_name)| {
            Ref { call, table_name }
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
        let Self { call, table_name } = self;
        if cont.is_none() {
            if let Some(table_id) = tn.get(&table_name) {
                let data_type = Record {
                    fields: HashMap::from([(
                        table_name,
                        RecordData::Scalar(ScalarType::Ref(*table_id)),
                    )]),
                    stream: true,
                };

                let out_edge = lp.operator_edges.insert(Edge::Null);

                let ref_op = lp.operators.insert(LogicalOperator {
                    query: Some(qk),
                    operator: LogicalOp::Scan {
                        access: TableAccess::Ref,
                        table: *table_id,
                        output: out_edge,
                    },
                });

                lp.operator_edges[out_edge] = Edge::Uni {
                    from: ref_op,
                    with: data_type.clone(),
                };

                Ok(StreamContext::Continue(Continue {
                    data_type,
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
