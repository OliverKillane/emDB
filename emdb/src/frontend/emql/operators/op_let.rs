use super::*;

#[derive(Debug)]
pub struct Let {
    call: Ident,
    var_name: Ident,
}

impl EMQLOperator for Let {
    const NAME: &'static str = "let";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(seq(matchident("let"), setrepr(getident(), "<var name>")), |(call, var_name)| Let {
            call,
            var_name,
        })
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
        let Self { call, var_name } = self;
        if let Some(prev_state) = cont {
            if let Some(varstate) = vs.get(&var_name) {
                Err(singlelist(match varstate {
                    VarState::Used { created, used } => {
                        errors::query_let_variable_already_assigned(
                            &var_name,
                            *created,
                            Some(*used),
                        )
                    }
                    VarState::Available { created, state } => {
                        errors::query_let_variable_already_assigned(&var_name, *created, None)
                    }
                }))
            } else {
                let var_span = var_name.span();
                vs.insert(
                    var_name,
                    VarState::Available {
                        created: var_span,
                        state: prev_state,
                    },
                );
                Ok(StreamContext::Nothing {
                    last_span: call.span(),
                })
            }
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
