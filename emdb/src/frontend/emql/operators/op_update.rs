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
        lp: &mut LogicalPlan,
        tn: &HashMap<Ident, TableKey>,
        qk: QueryKey,
        vs: &mut HashMap<Ident, VarState>,
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
            let raw_table_id = match prev.data_type.fields.get(&reference) {
                Some(RecordData::Scalar(ScalarType::Ref(table))) => Some(table),
                Some(d) => {
                    errors.push_back(errors::query_expected_reference_type_for_update(
                        lp, d, &reference,
                    ));
                    None
                }
                None => {
                    errors.push_back(errors::query_update_reference_not_present(
                        lp,
                        &reference,
                        prev.last_span,
                        &prev.data_type,
                    ));
                    None
                }
            };

            if let (Some(table_id), nondup_fields) = (raw_table_id, raw_fields) {
                // TODO: we could check the non-duplicate fields, cost/benefit unclear

                let table = lp.tables.get(*table_id).unwrap();

                for (id, _) in nondup_fields.iter() {
                    if !table.columns.contains_key(id) {
                        errors.push_back(errors::query_update_field_not_in_table(id, &table.name));
                    }
                }

                if errors.is_empty() {
                    let out_edge = lp.operator_edges.insert(Edge::Null);
                    let update_op = lp.operators.insert(LogicalOperator {
                        query: Some(qk),
                        operator: LogicalOp::Update {
                            input: prev.prev_edge,
                            reference: reference.clone(),
                            table: *table_id,
                            mapping: nondup_fields,
                            output: out_edge,
                        },
                    });

                    let out_data_type = Record {
                        fields: HashMap::from([(
                            reference.clone(),
                            RecordData::Scalar(ScalarType::Ref(*table_id)),
                        )]),
                        stream: prev.data_type.stream,
                    };

                    lp.operator_edges[out_edge] = Edge::Uni {
                        from: update_op,
                        with: out_data_type.clone(),
                    };

                    Ok(StreamContext::Continue(Continue {
                        data_type: out_data_type,
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
