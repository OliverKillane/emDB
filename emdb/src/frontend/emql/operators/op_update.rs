use super::*;

#[derive(Debug)]
pub struct Update {
    call: Ident,
    reference: Ident,
    fields: Vec<(Ident, Expr)>,
}

impl EMQLOperator for Update {
    const NAME: &'static str = "update";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                seqs!(setrepr(getident(), "<table>"), matchident("use"), fields_expr()),
            ),
            |(call, (reference, (_, fields)))| Update {
                call,
                reference,
                fields,
            },
        )
    }

    fn build_logical(
        self,
        lp: &mut plan::Plan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        op_ctx: plan::Key<plan::Context>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self {
            call,
            reference,
            fields,
        } = self;
        if let Some(cont) = cont {
            linear_builder(
                lp,
                qk,
                op_ctx,
                cont,
                |lp, op_ctx, prev, next_edge| {
                    let (raw_fields, mut errors) = extract_fields(fields, errors::query_operator_field_redefined);
                    
                    let raw_table_id = if let Some(sk) = lp.get_record_type(prev.data_type.fields).fields.get(&reference) {
                        if let plan::ScalarTypeConc::TableRef(table) = lp.get_scalar_type(*sk) { Some(*table) } else {
                                errors.push_back(errors::query_expected_reference_type_for_update(
                                    lp, sk, &reference,
                                ));
                                None
                            }
                    } else {
                        errors.push_back(errors::query_update_reference_not_present(
                            lp,
                            &reference,
                            prev.last_span,
                            &prev.data_type.fields,
                        ));
                        None
                    };

                    if let (Some(table_id), nondup_fields) = (raw_table_id, raw_fields) {
                        let table = lp.get_table(table_id);

                        for id in nondup_fields.keys() {
                            if !table.columns.contains_key(id) {
                                errors.push_back(errors::query_update_field_not_in_table(id, &table.name));
                            }
                        }

                        if errors.is_empty() {
                            Ok(
                                LinearBuilderState { 
                                    data_out: prev.data_type, 
                                    op: plan::Operator::Modify (plan::Update { 
                                            input: prev.prev_edge,
                                            reference: reference.clone(),
                                            table: table_id,
                                            mapping: nondup_fields,
                                            output: next_edge, 
                                        }.into()
                                    ), 
                                call_span: call.span()}
                            )
                        } else {
                            Err(errors)
                        }
                    } else {
                        Err(errors)
                    }
                }
            )
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
