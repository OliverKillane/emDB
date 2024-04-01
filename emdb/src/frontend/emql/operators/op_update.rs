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
                seqs!(getident(), matchident("use"), fields_expr()),
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
            fields,
        } = self;
        if let Some(prev) = cont {
            let (raw_fields, mut errors) =
                extract_fields(fields, errors::query_operator_field_redefined);

            // get the table to update
            let raw_table_id = if let Some(sk) = lp.get_record_type(prev.data_type.fields).fields.get(&reference) {
                match lp.get_scalar_type(*sk) {
                    plan::ScalarTypeConc::TableRef(table) => Some(*table),
                    _ => {
                        errors.push_back(errors::query_expected_reference_type_for_update(
                            lp, sk, &reference,
                        ));
                        None
                    }
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
                // TODO: we could check the non-duplicate fields, cost/benefit unclear

                let table = lp.get_table(table_id);

                for (id, _) in nondup_fields.iter() {
                    if !table.columns.contains_key(id) {
                        errors.push_back(errors::query_update_field_not_in_table(id, &table.name));
                    }
                }

                if errors.is_empty() {
                    let out_edge = lp.operator_edges.insert(plan::DataFlow::Null);
                    let update_op = lp.operators.insert(
                        plan::Operator { query: qk, kind: plan::OperatorKind::Modify { modify_after: mo.clone(), op: plan::ModifyOperator::Update { 
                            input: prev.prev_edge,
                            reference: reference.clone(),
                            table: table_id,
                            mapping: nondup_fields,
                            output: out_edge, } } }
                    );
                    *mo = Some(update_op);

                    lp.operator_edges[out_edge] = plan::DataFlow::Incomplete {
                        from: update_op,
                        with: prev.data_type.clone(),
                    };

                    Ok(StreamContext::Continue(Continue {
                        data_type: prev.data_type,
                        prev_edge: out_edge,
                        last_span: call.span(),
                    }))
                } else {
                    Err(errors)
                }
            } else {
                Err(errors)
            }
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
