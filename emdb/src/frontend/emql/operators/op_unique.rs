use super::*;

#[derive(Debug)]
pub struct Unique {
    call: Ident,
    table: Ident,
    refs: bool,
    unique_field: Ident,
    from_expr: Expr,
}

impl EMQLOperator for Unique {
    const NAME: &'static str = "unique";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                seqs!(
                    choices!(
                        peekident("ref") => mapsuc(matchident("ref"), |_| true),
                        otherwise => mapsuc(nothing(), |_|false)
                    ),
                    getident(),
                    matchident("for"),
                    getident(),
                    matchident("as"),
                    syn(collectuntil(isempty()))
                ),
            ),
            |(call, (refs, (table, (_, (unique_field, (_, from_expr))))))| Unique {
                call,
                table,
                refs,
                unique_field,
                from_expr,
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
            table,
            refs,
            unique_field,
            from_expr,
        } = self;
        if cont.is_none() {
            if let Some(table_id) = tn.get(&table) {
                let out_edge = lp.operator_edges.insert(Edge::Null);

                let logical_table = lp.tables.get(*table_id).unwrap();

                if let Some(using_col) = logical_table.columns.get(&unique_field) {
                    if let UniqueCons::Unique(_) = using_col.constraints.unique {
                        let (unique_op, data_type) = if refs {
                            (
                                lp.operators.insert(LogicalOperator {
                                    query: Some(qk),
                                    operator: LogicalOp::Unique {
                                        unique_field,
                                        access: TableAccess::Ref,
                                        from_expr,
                                        table: *table_id,
                                        output: out_edge,
                                    },
                                }),
                                Record {
                                    fields: HashMap::from([(
                                        table,
                                        RecordData::Scalar(ScalarType::Ref(*table_id)),
                                    )]),
                                    stream: false,
                                },
                            )
                        } else {
                            (
                                lp.operators.insert(LogicalOperator {
                                    query: Some(qk),
                                    operator: LogicalOp::Unique {
                                        unique_field,
                                        access: TableAccess::AllCols,
                                        from_expr,
                                        table: *table_id,
                                        output: out_edge,
                                    },
                                }),
                                logical_table.get_all_cols_type(),
                            )
                        };

                        lp.operator_edges[out_edge] = Edge::Uni {
                            from: unique_op,
                            with: data_type.clone(),
                        };

                        Ok(StreamContext::Continue(Continue {
                            data_type,
                            prev_edge: out_edge,
                            last_span: call.span(),
                        }))
                    } else {
                        Err(singlelist(errors::query_unique_field_is_not_unique(
                            &unique_field,
                            &logical_table.name,
                        )))
                    }
                } else {
                    Err(singlelist(errors::query_unique_no_field_in_table(
                        &unique_field,
                        &logical_table.name,
                    )))
                }
            } else {
                Err(singlelist(errors::query_unique_table_not_found(&table)))
            }
        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
