use super::*;
use super::super::sem::ast_typeto_scalar;

#[derive(Debug)]
pub struct Row {
    call: Ident,
    fields: Vec<(Ident, (AstType, Expr))>,
}

impl EMQLOperator for Row {
    const NAME: &'static str = "row";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, fields_assign()),
            |(call, fields)| Row { call, fields },
        )
    }

    fn build_logical(
        self,
        lp: &mut LogicalPlan,
        tn: &HashMap<Ident, TableKey>,
        qk: QueryKey,
        vs: &mut HashMap<Ident, VarState>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, fields } = self;
        if cont.is_none() {
            let (fields, mut errors) = extract_fields(fields, errors::query_operator_field_redefined);
            
            let mut type_fields = HashMap::new();
            let mut expr_fields = HashMap::new();

            for (field, (dt, expr)) in fields.into_iter().filter_map(|(field, (ast_type, expr))| {let dt = RecordData::Scalar(ast_typeto_scalar(tn, ast_type, &mut errors, |e| errors::query_nonexistent_table(&call, e))?); Some((field, (dt, expr)))} ) {
                type_fields.insert(field.clone(), dt);
                expr_fields.insert(field, expr);
            }

            
            if errors.is_empty() {
                let dt = Record { fields: type_fields, stream: false };

                let out_edge = lp.operator_edges.insert(Edge::Null);
                let row_op = lp.operators.insert(LogicalOperator { query: Some(qk), operator: LogicalOp::Row { fields: expr_fields, output: out_edge } });
                lp.operator_edges[out_edge] = Edge::Uni { from: row_op, with: dt.clone() };

                Ok(
                    StreamContext::Continue(Continue {
                        data_type: dt,
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
