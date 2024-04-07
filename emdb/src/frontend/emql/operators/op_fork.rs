use combi::tokens::derived::listsep;

use super::*;

#[derive(Debug)]
pub struct Fork {
    call: Ident,
    vars: Vec<Ident>,
}

impl EMQLOperator for Fork {
    const NAME: &'static str = "fork";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, 
                seq(
                    matchident("let"),
                    listsep(',', setrepr(getident(), "<variable>"))
                )
            ),
            |(call, (_, vars))| Fork {call, vars}
        )
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
        let Self { call, vars } = self;
        if let Some(cont) = cont {
            let mut errors = LinkedList::new();
            for var in vars.iter() {
                if let Some(varstate) = vs.get(var) {
                    errors.push_back(match varstate {
                        VarState::Used { created, used } => {
                            errors::query_let_variable_already_assigned(
                                var,
                                *created,
                                Some(*used),
                            )
                        }
                        VarState::Available { created, state } => {
                            errors::query_let_variable_already_assigned(var, *created, None)
                        }
                    })
                }
            }

            if errors.is_empty() {
                let var_edges: Vec<plan::Key<plan::DataFlow>> = vars.into_iter().map(
                    |var| {
                        let out_edge = lp.dataflow.insert(plan::DataFlow::Null);
                        vs.insert(var.clone(), VarState::Available {
                            created: var.span(),
                            state: Continue { data_type: cont.data_type.clone(), prev_edge: out_edge, last_span: call.span() }
                        });
                        out_edge
                    }
                ).collect();

                let fork_op = lp.operators.insert(
                    plan::Operator {
                        query: qk,
                        kind: plan::OperatorKind::Pure(plan::Fork { input: cont.prev_edge, outputs: var_edges.clone() }.into()),
                    }
                );

                for edge in var_edges {
                    *lp.get_mut_dataflow(edge) = plan::DataFlow::Incomplete { from: fork_op, with: cont.data_type.clone() }
                }

                update_incomplete(lp.get_mut_dataflow(cont.prev_edge), fork_op);

                Ok(StreamContext::Nothing { last_span: call.span() }) 

            } else {
                Err(errors)
            }
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}