use super::*;

#[derive(Debug)]
pub struct Row {
    call: Ident,
    fields: Vec<(Ident, (AstType, Expr))>,
}

impl EMQLOperator for Row {
    const NAME: &'static str = "row";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, fields_assign()),
            |(call, fields)| Row { call, fields },
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
        let Self { call, fields } = self;
        if cont.is_none() {
            let (fields, mut errors) = extract_fields_ordered(fields, errors::query_operator_field_redefined);
            
            let mut type_fields = HashMap::new();
            let mut expr_fields = Vec::new();

            for (field, (ast_type, expr)) in fields {
                match query_ast_typeto_scalar(tn, ts, ast_type, |e| errors::query_nonexistent_table(&call, e), errors::query_no_cust_type_found) {
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
                let data = plan::Data {
                    fields: lp.record_types.insert(plan::ConcRef::Conc(plan::RecordConc { fields: type_fields })),
                    stream: false,
                };

                let out_edge = lp.dataflow.insert(plan::DataFlow::Null);
                let row_op = lp.operators.insert(plan::Row { fields: expr_fields, output: out_edge  }.into());
                
                *lp.get_mut_dataflow(out_edge) = plan::DataFlow::Incomplete { from: row_op, with: data.clone() };
                lp.get_mut_context(op_ctx).add_operator(row_op);
                Ok(
                    StreamContext::Continue(Continue {
                        data_type: data,
                        prev_edge: out_edge,
                        last_span: call.span(),
                    })
                )
            } else {
                Err(errors)
            }
        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
