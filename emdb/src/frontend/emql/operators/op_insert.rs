//! Insert record into a table
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
                setrepr(getident(), "<table to insert into>"),
                matchident("as"),
                matchident("ref"),
                setrepr(getident(), "<name of refs to return>")
            )),
            |(call, (table_name, (_, (_, out_ref))))| Insert { call, table_name, out_ref },
        )
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
        let Self { call, table_name, out_ref } = self;
        if let Some(cont) = cont {
            linear_builder(
                lp,
                qk,
                mo,
                cont,
                |lp, mo, Continue {
                    data_type: plan::Data { fields, stream },
                    prev_edge,
                    last_span,
                }, next_edge | {
                    if let Some(table_id) = tn.get(&table_name) {
                        let table = lp.get_table(*table_id);
                        let insert_access = generate_access(*table_id, plan::TableAccess::AllCols, lp, None).unwrap();
                        if plan::record_type_eq(lp, &insert_access, &fields) {
                            let table_ref_t = lp.scalar_types.insert(plan::ConcRef::Conc(plan::ScalarTypeConc::TableRef(*table_id)));
                            let out_data_type = lp.record_types.insert(plan::ConcRef::Conc(plan::RecordConc {fields: HashMap::from([(out_ref.clone(), table_ref_t)])}));
                            let out_data = plan::Data {fields: out_data_type, stream };
                            Ok(
                                LinearBuilderState {
                                    data_out: plan::Data {
                                        fields: out_data_type,
                                        stream,
                                    },
                                    op_kind: plan::OperatorKind::Modify { modify_after: *mo, op: plan::ModifyOperator::Insert {  input: prev_edge,
                                        table: *table_id,
                                        out_ref,
                                        output: next_edge, } },
                                    call_span: call.span(),
                                    update_mo: true,
                                }
                            )
                        } else {
                            Err(singlelist(errors::query_invalid_record_type(lp, &call, last_span, &insert_access, &fields)))
                        }
                    } else {
                        Err(singlelist(errors::query_nonexistent_table(
                            &call,
                            &table_name,
                        )))
                    }
                }
            )
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
