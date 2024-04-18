use super::*;

#[derive(Debug)]
pub struct Unique {
    call: Ident,
    from: Ident,
    table: Ident,
    field: Ident,
    out: Ident,
}

impl EMQLOperator for Unique {
    const NAME: &'static str = "unique";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                seqs!(
                    setrepr(getident(), "<field to access with>"),
                    matchident("for"),
                    setrepr(getident(), "<table to access>"),
                    matchpunct('.'),
                    setrepr(getident(), "<unique field in table>"),
                    matchident("as"),
                    matchident("ref"),
                    setrepr(getident(), "<field to put reference in>")
                ),
            ),
            |(call,  (from, (_, (table, (_, (field, (_, (_, out))))))) )| Unique {
                call,
                from,
                field,
                table,
                out,
            },
        )
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
        let Self {
            call,
            from,
            table,
            field,
            out,
        } = self;
        

        if let Some(cont) = cont {
            linear_builder(
                lp,
                qk,
                op_ctx,
                cont,
                |lp, op_ctx, Continue { data_type, prev_edge, last_span }, next_edge| {
                    let rec_from = from.clone().into();
                    let rec_field = field.clone().into();
                    let rec_out = out.clone().into();
                    if let Some(table_id) = tn.get(&table) {
                        let table = lp.get_table(*table_id);
                        if let Some(using_col) = table.columns.get(&field) {
                            if let Some(plan::Constraint { alias, cons: plan::Unique }) = &using_col.cons.unique {
                                let record_type = generate_access::unique(*table_id, out.clone(), lp, data_type.fields)?;
                                Ok(
                                    LinearBuilderState {
                                        data_out: plan::Data{ fields: record_type, stream: false }, 
                                        op: plan::GetUnique { 
                                            input: prev_edge, from: rec_from, table: *table_id, field: rec_field, out: rec_out, output: next_edge }.into(), 
                                        call_span: call.span()
                                    }
                                )
                            } else {
                                Err(singlelist(errors::query_unique_field_is_not_unique(
                                    &field,
                                    &table.name,
                                )))
                            }
                        } else {
                            Err(singlelist(errors::query_unique_no_field_in_table(
                                &field,
                                &table.name,
                            )))
                        }
                    } else {
                        Err(singlelist(errors::query_unique_table_not_found(&table)))
                    }
                }
            )
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
