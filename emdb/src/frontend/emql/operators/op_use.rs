/// Use is a significant syntactic sugar
/// - Allows for `scanref <table> |> deref |> expand` to be written as `use <table>`
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

                let ref_field = plan::RecordField::Internal(0);
                let rec_field = plan::RecordField::Internal(1);

                let table_fields_type = lp.record_types.insert(get_all_cols(lp, *table_id).into());
                let table_fields_scalar_type = lp.scalar_types.insert(plan::ScalarTypeConc::Record(table_fields_type).into());

                let scanref_cont = create_scanref(lp, op_ctx, *table_id, ref_field.clone(), call.span());
                
                let deref_access = valid_linear_builder(lp, qk, op_ctx, scanref_cont, 
                    |lp, op_ctx, Continue {
                        data_type,
                        prev_edge,
                        last_span,
                    }, next_edge| {
                        LinearBuilderState {
                            data_out: plan::Data {
                                fields: lp.record_types.insert(plan::RecordConc{ fields: HashMap::from([(rec_field.clone(), table_fields_scalar_type)]) }.into()),
                                stream: true,
                            },
                            op: plan::DeRef { input: prev_edge, reference: ref_field, named: rec_field.clone(), table: *table_id, output: next_edge }.into(),
                            call_span: call.span(),
                        }
                    }
                );

                let expand_access = valid_linear_builder(lp, qk, op_ctx, deref_access, |lp, op_ctx, Continue {
                    data_type,
                    prev_edge,
                    last_span,
                }, next_edge| {
                    LinearBuilderState {
                        data_out: plan::Data {
                            fields: table_fields_type,
                            stream: true,
                        },
                        op: plan::Expand {
                            input: prev_edge,
                            field: rec_field,
                            output: next_edge,
                        }.into(),
                        call_span: call.span(),
                    }
                });
                
                Ok(StreamContext::Continue(expand_access))
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
