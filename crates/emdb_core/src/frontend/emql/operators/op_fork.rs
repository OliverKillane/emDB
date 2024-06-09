use combi::tokens::derived::listsep;

use super::*;

#[derive(Debug)]
pub struct Fork {
    call: Ident,
    vars: Vec<Ident>,
}

impl EMQLOperator for Fork {
    const NAME: &'static str = "fork";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                seq(
                    matchident("let"),
                    listsep(',', setrepr(getident(), "<variable>")),
                ),
            ),
            |(call, (_, vars))| Fork { call, vars },
        )
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
        let Self { call, vars } = self;
        if let Some(cont) = cont {
            let mut errors = LinkedList::new();


            let (var_edges, vars_added): (Vec<plan::Key<plan::DataFlow>>, Vec<Ident>) = vars
                .into_iter()
                .filter_map(|var| {
                    let out_edge = lp.dataflow.insert(plan::DataFlow::Null);
                    if assign_new_var(
                        var.clone(),
                        Continue {
                            data_type: cont.data_type.clone(),
                            prev_edge: out_edge,
                            last_span: call.span(),
                        },
                        vs,
                        tn,
                        &mut errors,
                    ) {
                        Some((out_edge, var))
                    } else {
                        lp.dataflow.remove(out_edge);
                        None
                    }
                })
                .unzip();

            if errors.is_empty() {
                let fork_op = lp.operators.insert(
                    plan::Fork {
                        input: cont.prev_edge,
                        outputs: var_edges.clone(),
                    }
                    .into(),
                );

                for edge in var_edges {
                    *lp.get_mut_dataflow(edge) = plan::DataFlow::Incomplete {
                        from: fork_op,
                        with: cont.data_type.clone(),
                    }
                }

                update_incomplete(lp.get_mut_dataflow(cont.prev_edge), fork_op);
                lp.get_mut_context(op_ctx).add_operator(fork_op);

                Ok(StreamContext::Nothing {
                    last_span: call.span(),
                })
            } else {
                // NOTE: given we were unable to add all the edge, we need to repair the 
                //       logical plan so that further semantic analysis can continue with 
                //       a valid plan.
                //       I previously implemented by checking names first, but:
                //       - fork can duplicate names
                //       - easier to change with one `assign_new_var` function to do all 
                //         variable assignment
                for var in vars_added {
                    vs.remove(&var);
                }
                for edge in var_edges {
                    lp.dataflow.remove(edge);
                }

                Err(errors)
            }
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
