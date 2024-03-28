use super::*;

#[derive(Debug)]
pub struct Use {
    call: Ident,
    var_name: Ident,
}

impl EMQLOperator for Use {
    const NAME: &'static str = "use";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(seq(matchident("use"), getident()), |(call, var_name)| Use {
            call,
            var_name,
        })
    }

    fn build_logical(
        self,
        lp: &mut LogicalPlan,
        tn: &HashMap<Ident, TableKey>,
        qk: QueryKey,
        vs: &mut HashMap<Ident, VarState>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, var_name } = self;
        if cont.is_none() {
            if let Some(table_id) = tn.get(&var_name) {
                let data_type = lp.tables.get(*table_id).unwrap().get_all_cols_type();
                let out_edge = lp.operator_edges.insert(Edge::Null);
                let use_op = lp.operators.insert(LogicalOperator {
                    query: Some(qk),
                    operator: LogicalOp::Scan {
                        access: TableAccess::AllCols,
                        table: *table_id,
                        output: out_edge,
                    },
                });
                lp.operator_edges[out_edge] = Edge::Uni {
                    from: use_op,
                    with: data_type.clone(),
                };

                Ok(StreamContext::Continue(Continue {
                    data_type,
                    prev_edge: out_edge,
                    last_span: call.span(),
                }))
            } else if let Some(var) = vs.get_mut(&var_name) {
                match var {
                    VarState::Used { created, used } => Err(singlelist(
                        errors::query_use_variable_already_used(&var_name, *created, *used),
                    )),
                    VarState::Available { created, state } => {
                        let ret = Ok(StreamContext::Continue(state.clone()));
                        *var = VarState::Used {
                            created: *created,
                            used: call.span(),
                        };
                        ret
                    }
                }
            } else {
                Err(singlelist(errors::query_invalid_use(&var_name)))
            }
        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
