use super::*;

#[derive(Debug)]
pub struct Use {
    call: Ident,
    var_name: Ident,
}

impl EMQLOperator for Use {
    const NAME: &'static str = "use";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(seq(matchident("use"), setrepr(getident(), "<table>")), |(call, var_name)| Use {
            call,
            var_name,
        })
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
        let Self { call, var_name } = self;
        if cont.is_none() {
            if let Some(table_id) = tn.get(&var_name) {
                let access = plan::TableAccess::AllCols;
                let record_type = generate_access(*table_id, access.clone(), lp, None).unwrap();
                let out_edge = lp.dataflow.insert(plan::DataFlow::Null);
                let use_op = lp.operators.insert(plan::Operator::Access (plan::Scan { access, table: *table_id, output: out_edge }.into()));
                let data_type = plan::Data { fields: record_type, stream: true };

                *lp.get_mut_dataflow(out_edge) = plan::DataFlow::Incomplete {
                    from: use_op,
                    with: data_type.clone(),
                };
                lp.get_mut_context(op_ctx).add_operator(use_op);

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
                Err(singlelist(errors::query_invalid_use(&var_name, tn, vs)))
            }
        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
