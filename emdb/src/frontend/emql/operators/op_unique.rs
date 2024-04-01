use self::plan::{generate_access, Data};

use super::*;

#[derive(Debug)]
pub struct Unique {
    call: Ident,
    table: Ident,
    refs: Option<Ident>,
    unique_field: Ident,
    from_expr: Expr,
}

impl EMQLOperator for Unique {
    const NAME: &'static str = "unique";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                seqs!(
                    choices!(
                        peekident("ref") => mapsuc(seq(matchident("ref"),getident()), |(_, ref_name)| Some(ref_name)),
                        otherwise => mapsuc(nothing(), |_|None)
                    ),
                    getident(),
                    matchident("for"),
                    getident(),
                    matchident("as"),
                    syn(collectuntil(isempty()))
                ),
            ),
            |(call, (refs, (table, (_, (unique_field, (_, from_expr))))))| Unique {
                call,
                table,
                refs,
                unique_field,
                from_expr,
            },
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
        let Self {
            call,
            table,
            refs,
            unique_field,
            from_expr,
        } = self;
        if cont.is_none() {
            if let Some(table_id) = tn.get(&table) {
                let out_edge = lp.operator_edges.insert(plan::DataFlow::Null);

                let logical_table = lp.get_table(*table_id);

                if let Some(using_col) = logical_table.columns.get(&unique_field) {

                    if let Some(plan::Constraint { alias, cons: plan::Unique }) = &using_col.cons.unique {
                    
                        let access = if let Some(ref_name) = refs {
                            plan::TableAccess::Ref(ref_name.clone())
                        } else {
                            plan::TableAccess::AllCols
                        };

                        let record_type = generate_access(*table_id, access.clone(), lp).unwrap();
                        let unique_op = lp.operators.insert(plan::Operator { query: qk, kind: plan::OperatorKind::Access { access_after: mo.clone(), op: plan::AccessOperator::Unique { 
                            unique_field, access, from_expr, table: *table_id, output: out_edge } } });
                        *mo = Some(unique_op);
                        let data_type = Data{ fields: record_type, stream: false };
                        lp.operator_edges[out_edge] = plan::DataFlow::Incomplete {
                            from: unique_op,
                            with: data_type.clone(),
                        };

                        Ok(StreamContext::Continue(Continue {
                            data_type,
                            prev_edge: out_edge,
                            last_span: call.span(),
                        }))
                    } else {
                        Err(singlelist(errors::query_unique_field_is_not_unique(
                            &unique_field,
                            &logical_table.name,
                        )))
                    }
                } else {
                    Err(singlelist(errors::query_unique_no_field_in_table(
                        &unique_field,
                        &logical_table.name,
                    )))
                }
            } else {
                Err(singlelist(errors::query_unique_table_not_found(&table)))
            }
        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
