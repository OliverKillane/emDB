//! Delete a row from a table by reference.
use super::*;

#[derive(Debug)]
pub struct Delete {
    call: Ident,
    field: Ident,
}

impl EMQLOperator for Delete {
    const NAME: &'static str = "delete";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(functional_style(Self::NAME, setrepr(getident(), "<row ref to delete>")), |(call, field)| {
            Delete { call, field }
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
        let Self { call, field } = self;
        if let Some(cont) = cont
        {
            linear_builder(
                lp,
                qk,
                mo,
                cont,
                |lp, mo, Continue {
                    data_type,
                    prev_edge,
                    last_span,
                }, next_edge| {
                    if let Some(ts) = lp.get_record_type(data_type.fields).fields.get(&field) {
                        if let plan::ScalarTypeConc::TableRef(table_id) = lp.get_scalar_type(*ts) {
                            Ok(
                                LinearBuilderState { 
                                    data_out: data_type, 
                                    op_kind: plan::OperatorKind::Modify { modify_after: *mo, op: plan::Delete { input: prev_edge, reference: field, table: *table_id, output: next_edge }.into() }, 
                                    call_span: call.span(), 
                                    update_mo: true 
                                }
                            )
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
                }
            )
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
