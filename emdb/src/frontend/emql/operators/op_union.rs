use combi::tokens::derived::listsep;

use super::*;

#[derive(Debug)]
pub struct Union {
    call: Ident,
    vars: Vec<Ident>
}

impl EMQLOperator for Union {
    const NAME: &'static str = "union";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(functional_style(Self::NAME, seq(
            matchident("use"),
            listsep(',', setrepr(getident(), "<variable>"))
        )), |(call, (_, vars))| Union {call, vars})
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
        if cont.is_none() {
            let mut errors = LinkedList::new();

            let mut var_info = vars.into_iter().filter_map(|var| {
                if let Some(varstate) = vs.get_mut(&var) {
                    match varstate {
                        VarState::Available { created, state } => {
                            let saved_state = state.clone();
                            *varstate = VarState::Used { created: *created, used: var.span() };
                            Some((var, saved_state))
                        },
                        VarState::Used { created, used  } => {
                            errors.push_back(errors::query_use_variable_already_used(&var, *created, *used));
                            None
                        },
                    }
                } else {
                    errors.push_back(errors::query_invalid_use(&var, tn, vs));
                    None
                }
            });

            
            if let Some((var_name, var_type)) = var_info.next() {
                if var_type.data_type.stream {
                    let out_data_type = var_type.data_type.fields;
                    let mut in_edges = vec![var_type.prev_edge];
                    let mut in_types = Vec::new();
                    let mut other_errors = LinkedList::new();

                    for (other_var, other_type) in var_info {
                        if !other_type.data_type.stream {
                            other_errors.push_back(errors::union_requires_streams(&call, &other_var));
                        }

                        if plan::record_type_eq(lp, &out_data_type, &other_type.data_type.fields) {
                            in_edges.push(other_type.prev_edge);
                            in_types.push(other_type.data_type.fields);
                        } else {
                            other_errors.push_back(errors::union_not_same_type(lp, &call, &var_name, &out_data_type, &other_var, &other_type.data_type.fields));
                        }
                    }

                    errors.append(&mut other_errors);

                    if errors.is_empty() {
                        // NOTE: we can coerce the types to be the same (index/object, not just 
                        //       equality), by making one reference the other. 
                        //       - equality => can coerce as (type, table ref) must be the same
                        //       - if equal, resetting is fine
                        //       - by changing the type, all references to that type everywhere are 
                        //         updated
                        //       - coersion means we can map each type to a rust type, and have the 
                        //         same implementation shared (e.g. when choosing `type foo`'s 
                        //         implementation )
                        for in_type in in_types {
                            plan::coerce_record_type(lp, out_data_type, in_type);
                        }
                        let out_edge = lp.dataflow.insert(plan::DataFlow::Null);
                        let union_op = lp.operators.insert(plan::Operator {
                            query: qk,
                            kind: plan::OperatorKind::Pure(plan::Union {
                                inputs: in_edges.clone(),
                                output: out_edge,
                            }.into()),
                        });
                        for in_edge in in_edges {
                            update_incomplete(lp.get_mut_dataflow(in_edge), union_op);
                        }
                        let data_t =  plan::Data { fields: out_data_type, stream: true };
                        *lp.get_mut_dataflow(out_edge) = plan::DataFlow::Incomplete { from: union_op, with: data_t.clone() };
                        return Ok(StreamContext::Continue(Continue { data_type: data_t, prev_edge: out_edge, last_span: call.span() }));
                    }
                } else {
                    errors.push_back(errors::union_requires_streams(&call, &var_name));
                }
            } else {
                errors.push_back(errors::union_requires_at_least_one_input(&call));
            }

            Err(errors)

        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
