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
        lp: &mut LogicalPlan,
        tn: &HashMap<Ident, TableKey>,
        qk: QueryKey,
        vs: &mut HashMap<Ident, VarState>,
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
            if let Some(field) = data_type.fields.get(&named) {
                Err(singlelist(errors::query_deref_field_already_exists(&named)))
            } else if let Some(field_type) = data_type.fields.get(&reference) {
                match field_type {
                    RecordData::Record(r) => Err(singlelist(
                        errors::query_deref_cannot_deref_record(lp, &reference, r),
                    )),
                    RecordData::Scalar(ScalarType::Rust(t)) => Err(singlelist(
                        errors::query_deref_cannot_deref_rust_type(&reference, t),
                    )),
                    RecordData::Scalar(ScalarType::Bag(t)) => Err(singlelist(
                        errors::query_deref_cannot_deref_bag_type(lp, &reference, t),
                    )),
                    RecordData::Scalar(ScalarType::Ref(table_id)) => {
                        let table_id_copy = *table_id;
                        let table_type = lp.tables.get(*table_id).unwrap().get_all_cols_type();

                        let Record { mut fields, stream } = data_type;
                        fields.insert(
                            named.clone(),
                            RecordData::Record(Record {
                                fields: table_type.fields,
                                stream: false,
                            }),
                        );

                        let new_type = Record { fields, stream };

                        let out_edge = lp.operator_edges.insert(Edge::Null);
                        let deref_op = lp.operators.insert(LogicalOperator {
                            query: Some(qk),
                            operator: LogicalOp::DeRef {
                                input: prev_edge,
                                reference,
                                named,
                                table: table_id_copy,
                                output: out_edge,
                            },
                        });

                        lp.operator_edges[out_edge] = Edge::Uni {
                            from: deref_op,
                            with: new_type.clone(),
                        };

                        Ok(StreamContext::Continue(Continue {
                            data_type: new_type,
                            prev_edge: out_edge,
                            last_span: call.span(),
                        }))
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
