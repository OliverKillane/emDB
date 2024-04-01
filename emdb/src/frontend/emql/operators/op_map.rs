use self::plan::RecordConc;

use super::*;
use super::super::sem::ast_typeto_scalar;

#[derive(Debug)]
pub struct Map {
    call: Ident,
    new_fields: Vec<(Ident, (AstType, Expr))>,
}

impl EMQLOperator for Map {
    const NAME: &'static str = "map";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, fields_assign()),
            |(call, new_fields)| Map { call, new_fields },
        )
    }

    fn build_logical(
        self,
        lp: &mut plan::LogicalPlan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        mo: &mut Option<plan::Key<plan::Operator>>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, new_fields } = self;
        if let Some(Continue { data_type, prev_edge, last_span }) = cont {
            let (fields, mut errors) = extract_fields(new_fields, errors::query_operator_field_redefined);
            
            let mut type_fields = HashMap::new();
            let mut expr_fields = HashMap::new();

            for (field, (ast_type, expr)) in fields.into_iter() {
                match ast_typeto_scalar(tn, ts, ast_type, |e| errors::query_nonexistent_table(&call, e), errors::query_no_cust_type_found) {
                    Ok(t) => {
                        let t_index = lp.scalar_types.insert(t);
                        type_fields.insert(field.clone(), t_index);
                        expr_fields.insert(field, expr);
                    },
                    Err(e) => {
                        errors.push_back(e);
                    }
                }
            }

            if errors.is_empty() {
                let data = plan::Data {
                    fields: lp.record_types.insert(plan::ConcRef::Conc(RecordConc { fields: type_fields })),
                    stream: data_type.stream,
                };

                let out_edge = lp.operator_edges.insert(plan::DataFlow::Null);
                
                let map_op = lp.operators.insert(plan::Operator {
                    query: qk,
                    kind: plan::OperatorKind::Pure(plan::PureOperator::Map { input: prev_edge, mapping: expr_fields, output: out_edge  }),
                });
                
                lp.operator_edges[out_edge] = plan::DataFlow::Incomplete { from: map_op, with: data.clone() };

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
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
