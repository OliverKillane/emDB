use super::*;

#[derive(Debug)]
pub struct Insert {
    call: Ident,
    table_name: Ident,
}

impl EMQLOperator for Insert {
    const NAME: &'static str = "insert";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, getident()),
            |(call, table_name)| Insert { call, table_name },
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
        let Self { call, table_name } = self;
        if let Some(Continue {
            data_type: Record { mut fields, stream },
            prev_edge,
            last_span,
        }) = cont
        {
            if let Some(table_id) = tn.get(&table_name) {
                let table = lp.tables.get(*table_id).unwrap();
                let mut errors = LinkedList::new();

                for (id, col) in table.columns.iter() {
                    match fields.remove(id) {
                        Some(RecordData::Scalar(ScalarType::Rust(r))) => {
                            if r != col.data_type {
                                errors.push_back(errors::query_insert_field_rust_type_mismatch(
                                    lp,
                                    &call,
                                    id,
                                    &r,
                                    &col.data_type,
                                    last_span,
                                ));
                            }
                        }
                        Some(
                            other
                            @ (RecordData::Scalar(ScalarType::Ref(_)) | RecordData::Record(_)),
                        ) => {
                            errors.push_back(errors::query_insert_field_type_mismatch(
                                lp,
                                &call,
                                id,
                                &other,
                                &col.data_type,
                                last_span,
                            ));
                        }
                        None => {
                            errors.push_back(errors::query_insert_field_missing(
                                &call,
                                &table.name,
                                id,
                                last_span,
                            ));
                        }
                    }
                }

                for (id, _) in fields.iter() {
                    errors.push_back(errors::query_insert_extra_field(&call, id, &table.name));
                }

                if errors.is_empty() {
                    let out_edge = lp.operator_edges.insert(Edge::Null);
                    let insert_op = lp.operators.insert(LogicalOperator {
                        query: Some(qk),
                        operator: LogicalOp::Insert {
                            input: prev_edge,
                            table: *table_id,
                            output: out_edge,
                        },
                    });

                    let out_data_type = Record {
                        fields: HashMap::from([(
                            table_name.clone(),
                            RecordData::Scalar(ScalarType::Ref(*table_id)),
                        )]),
                        stream,
                    };

                    lp.operator_edges[out_edge] = Edge::Uni {
                        from: insert_op,
                        with: out_data_type.clone(),
                    };

                    Ok(StreamContext::Continue(Continue {
                        data_type: out_data_type,
                        prev_edge: out_edge,
                        last_span: table_name.span(),
                    }))
                } else {
                    Err(errors)
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
