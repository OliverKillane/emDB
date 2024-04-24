use super::*;

#[derive(Debug)]
pub struct Map {
    call: Ident,
    new_fields: Vec<(Ident, (AstType, Expr))>,
}

impl EMQLOperator for Map {
    const NAME: &'static str = "map";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, fields_assign()),
            |(call, new_fields)| Map { call, new_fields },
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
        let Self { call, new_fields } = self;
        if let Some(cont) = cont {
            linear_builder(
                lp,
                op_ctx,
                cont,
                |lp, op_ctx, Continue { data_type, prev_edge, last_span }, next_edge| {
                    let (fields, mut errors) = extract_fields(new_fields, errors::query_operator_field_redefined);
                    let mut type_fields = HashMap::new();
                    let mut expr_fields = Vec::new();

                    for (field, (ast_type, expr)) in fields {
                        match ast_typeto_scalar(tn, ts, ast_type, |e| errors::query_nonexistent_table(&call, e), errors::query_no_cust_type_found) {
                            Ok(t) => {
                                let t_index = lp.scalar_types.insert(t);
                                type_fields.insert(field.clone().into(), t_index);
                                expr_fields.push((field.into(), expr));
                            },
                            Err(e) => {
                                errors.push_back(e);
                            }
                        }
                    }

                    if errors.is_empty() {
                        Ok(
                            LinearBuilderState {
                                data_out: plan::Data {
                                    fields: lp.record_types.insert(plan::ConcRef::Conc(plan::RecordConc { fields: type_fields })),
                                    stream: data_type.stream,
                                },
                                op: plan::Map { input: prev_edge, mapping: expr_fields, output: next_edge  }.into(),
                                call_span: call.span()
                            }
                        )
                    } else {
                        Err(errors)
                    }
                }
            )
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
