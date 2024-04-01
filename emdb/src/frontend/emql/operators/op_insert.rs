use self::plan::RecordConc;

use super::*;

#[derive(Debug)]
pub struct Insert {
    call: Ident,
    table_name: Ident,
    out_ref: Ident,
}

impl EMQLOperator for Insert {
    const NAME: &'static str = "insert";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, seqs!(
                getident(),
                matchident("as"),
                matchident("ref"),
                getident()
            )),
            |(call, (table_name, (_, (_, out_ref))))| Insert { call, table_name, out_ref },
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
        let Self { call, table_name, out_ref } = self;
        if let Some(Continue {
            data_type: plan::Data { fields, stream },
            prev_edge,
            last_span,
        }) = cont
        {
            if let Some(table_id) = tn.get(&table_name) {
                let table = lp.get_table(*table_id);
                let insert_access = plan::generate_access(*table_id, plan::TableAccess::AllCols, lp).unwrap();
                if plan::record_type_eq(lp, &insert_access, &fields) {
                    let out_edge = lp.operator_edges.insert(plan::DataFlow::Null);
                    let insert_op = lp.operators.insert(plan::Operator { query: qk, kind: plan::OperatorKind::Modify { modify_after: mo.clone(), op: plan::ModifyOperator::Insert {  input: prev_edge,
                        table: *table_id,
                        out_ref: out_ref.clone(),
                        output: out_edge, } } });
                    
                    *mo = Some(insert_op.clone());
                    let table_ref_t = lp.scalar_types.insert(plan::ConcRef::Conc(plan::ScalarTypeConc::TableRef(*table_id)));
                    let out_data_type = lp.record_types.insert(plan::ConcRef::Conc(RecordConc {fields: HashMap::from([(out_ref, table_ref_t)])}));
                    let out_data = plan::Data {
                        fields: out_data_type,
                        stream,
                    };
                    lp.operator_edges[out_edge] = plan::DataFlow::Incomplete {
                        from: insert_op,
                        with: out_data.clone(),
                    };

                    Ok(StreamContext::Continue(Continue {
                        data_type: out_data,
                        prev_edge: out_edge,
                        last_span: table_name.span(),
                    }))
                } else {
                    Err(singlelist(errors::query_invalid_record_type(lp, &call, last_span, &insert_access, &fields)))
                }
            } else {
                Err(singlelist(errors::query_nonexistent_table(
                    &call,
                    &table_name,
                )))
            }
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
