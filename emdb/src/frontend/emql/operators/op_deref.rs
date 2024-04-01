use self::plan::{DataFlow, Record, RecordConc, ScalarTypeConc};

use super::*;

#[derive(Debug)]
pub struct DeRef {
    call: Ident,
    reference: Ident,
    named: Ident,
}

impl EMQLOperator for DeRef {
    const NAME: &'static str = "deref";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, seqs!(getident(), matchident("as"), getident())),
            |(call, (reference, (_, named)))| DeRef {
                call,
                reference,
                named,
            },
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
        let Self {
            call,
            reference,
            named,
        } = self;

        if let Some(Continue {
            data_type,
            prev_edge,
            last_span,
        }) = cont
        {
            // TODO: use append_field
            let rec_fields = &lp.get_record_type(data_type.fields).fields;
            if let Some(field) = rec_fields.get(&named) {
                Err(singlelist(errors::query_deref_field_already_exists(&named)))
            } else if let Some(field_type) = rec_fields.get(&reference) {
                match lp.get_scalar_type(*field_type) {
                    ScalarTypeConc::Record(r) => Err(singlelist(
                        errors::query_deref_cannot_deref_record(lp, &reference, r),
                    )),
                    ScalarTypeConc::Rust(rt) => Err(singlelist(
                        errors::query_deref_cannot_deref_rust_type(&reference, rt),
                    )),
                    ScalarTypeConc::Bag(b) => Err(singlelist(
                        errors::query_deref_cannot_deref_bag_type(lp, &reference, b),
                    )),
                    ScalarTypeConc::TableRef(table_id) => {
                        
                        let table_id_copy = *table_id;
                        let access = plan::TableAccess::AllCols;
                        let table_name = lp.get_table(*table_id).name.clone();
                        match plan::generate_access(*table_id, access.clone(), lp) {
                            Ok(dt) => {
                                let plan::RecordConc {mut fields} = lp.get_record_type(data_type.fields).clone();
                                let scalar_t = lp.scalar_types.insert(plan::ConcRef::Conc(ScalarTypeConc::Record(dt)));
                                
                                fields.insert(named.clone(), scalar_t);

                                let new_type = plan::Data { fields: lp.record_types.insert(plan::ConcRef::Conc(
                                    RecordConc { fields }
                                )), stream: data_type.stream } ;

                                let out_edge = lp.operator_edges.insert(plan::DataFlow::Null);
                                let deref_op = lp.operators.insert(plan::Operator { query: qk, kind: plan::OperatorKind::Access { access_after: mo.clone(), op: plan::AccessOperator::DeRef { 
                                    input: prev_edge, reference, access , named, table: table_id_copy, output: out_edge } } });
                                *mo = Some(deref_op.clone());
                                lp.operator_edges[out_edge] = plan::DataFlow::Incomplete {
                                    from: deref_op,
                                    with: new_type.clone(),
                                };

                                Ok(StreamContext::Continue(Continue {
                                    data_type: new_type,
                                    prev_edge: out_edge,
                                    last_span: call.span(),
                                }))

                            },
                            Err(ids) => {
                                Err(ids.into_iter().map(|id| errors::query_table_access_nonexisted_columns(&table_name, &id)).collect())
                            }
                        }
                    }
                }
            } else {
                Err(singlelist(errors::query_reference_field_missing(
                    &reference,
                )))
            }
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
