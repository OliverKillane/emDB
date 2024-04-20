//! Delete a row from a table by reference.
use super::*;

#[derive(Debug)]
pub struct Delete {
    call: Ident,
    field: Ident,
}

impl EMQLOperator for Delete {
    const NAME: &'static str = "delete";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(functional_style(Self::NAME, setrepr(getident(), "<row ref to delete>")), |(call, field)| {
            Delete { call, field }
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
        let Self { call, field } = self;
        if let Some(cont) = cont
        {
            linear_builder(
                lp,
                op_ctx,
                cont,
                |lp, mo, Continue {
                    data_type,
                    prev_edge,
                    last_span,
                }, next_edge| {
                    let rec_field = field.into();
                    if let Some(ts) = lp.get_record_type(data_type.fields).fields.get(&rec_field) {
                        if let plan::ScalarTypeConc::TableRef(table_id) = lp.get_scalar_type(*ts) {
                            Ok(
                                LinearBuilderState { 
                                    data_out: data_type, 
                                    op: plan::Delete { input: prev_edge, reference: rec_field, table: *table_id, output: next_edge }.into(), 
                                    call_span: call.span()
                                }
                            )
                        } else {
                            Err(singlelist(errors::query_delete_field_not_reference(
                                lp, &call, rec_field.get_field(), ts,
                            )))
                        }
                    } else {
                        Err(singlelist(errors::query_delete_field_not_present(
                            &call, rec_field.get_field(),
                        )))
                    }
                }
            )
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
