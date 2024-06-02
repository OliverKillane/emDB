//! Dereference a table reference to load the row
use super::*;

#[derive(Debug)]
pub struct DeRef {
    call: Ident,
    reference: Ident,
    named: Ident,
}

impl EMQLOperator for DeRef {
    const NAME: &'static str = "deref";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, seqs!(
                setrepr(getident(), "<field containing ref>"), 
                matchident("as"), 
                setrepr(getident(), "<new field to copy row to>"))),
            |(call, (reference, (_, named)))| DeRef {
                call,
                reference,
                named,
            },
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
        let Self {
            call,
            reference,
            named,
        } = self;

        if let Some(cont) = cont
        {
            linear_builder(
                lp,
                op_ctx,
                cont,
                |lp, op_ctx, Continue {
                    data_type,
                    prev_edge,
                    last_span,
                }, next_edge| {
                    let rec_fields = &lp.get_record_type_conc(data_type.fields).fields;
                    let rec_reference = reference.clone().into();
                    let rec_named = named.clone().into();
                    if let Some((existing, _)) = rec_fields.get_key_value(&rec_named) {
                        Err(singlelist(errors::query_deref_field_already_exists(&named, existing.get_field())))
                    } else if let Some(field_type) = rec_fields.get(&rec_reference) {
                        match lp.get_scalar_type_conc(*field_type) {
                            plan::ScalarTypeConc::Record(r) => Err(singlelist(
                                errors::query_deref_cannot_deref_record(lp, &reference, r),
                            )),
                            plan::ScalarTypeConc::Rust { ty, .. } => Err(singlelist(
                                errors::query_deref_cannot_deref_rust_type(&reference, ty),
                            )),
                            plan::ScalarTypeConc::Bag(b) => Err(singlelist(
                                errors::query_deref_cannot_deref_bag_type(lp, &reference, b),
                            )),
                            
                            plan::ScalarTypeConc::TableGet { table, field } => Err(singlelist(
                                errors::query_deref_cannot_deref_table_get(lp, &reference, *table, field)
                            )),
                            plan::ScalarTypeConc::TableRef(table_id) => {
                                let table_id_copy = *table_id;
                                let table_name = lp.get_table(*table_id).name.clone();
                                let generate_access::DereferenceTypes{outer_record: dt, inner_record} = generate_access::dereference(*table_id, lp, named, data_type.fields)?;                                
                                let new_type = plan::Data { fields: dt, stream: data_type.stream };
    
                                Ok(
                                    LinearBuilderState {
                                        data_out: new_type, 
                                        op: plan::DeRef { 
                                            input: prev_edge, reference: rec_reference, named: rec_named, table: table_id_copy, output: next_edge,
                                            named_type: inner_record, }.into(), 
                                        call_span: call.span()
                                    }
                                )
                            }
                        }
                    } else {
                        Err(singlelist(errors::query_reference_field_missing(
                            &reference,
                        )))
                    }
                }
            )
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
