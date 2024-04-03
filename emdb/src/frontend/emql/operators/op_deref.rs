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

    fn build_parser() -> impl TokenParser<Self> {
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
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        mo: &mut Option<plan::Key<plan::Operator>>,
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
                qk,
                mo,
                cont,
                |lp, mo, Continue {
                    data_type,
                    prev_edge,
                    last_span,
                }, next_edge| {
                    let rec_fields = &lp.get_record_type(data_type.fields).fields;
                    if let Some((existing, _)) = rec_fields.get_key_value(&named) {
                        // TODO: use append_field
                        Err(singlelist(errors::query_deref_field_already_exists(&named, existing)))
                    } else if let Some(field_type) = rec_fields.get(&reference) {
                        match lp.get_scalar_type(*field_type) {
                            plan::ScalarTypeConc::Record(r) => Err(singlelist(
                                errors::query_deref_cannot_deref_record(lp, &reference, r),
                            )),
                            plan::ScalarTypeConc::Rust(rt) => Err(singlelist(
                                errors::query_deref_cannot_deref_rust_type(&reference, rt),
                            )),
                            plan::ScalarTypeConc::Bag(b) => Err(singlelist(
                                errors::query_deref_cannot_deref_bag_type(lp, &reference, b),
                            )),
                            plan::ScalarTypeConc::TableRef(table_id) => {
                                
                                let table_id_copy = *table_id;
                                let access = plan::TableAccess::AllCols;
                                let table_name = lp.get_table(*table_id).name.clone();
                                let dt = generate_access(*table_id, access.clone(), lp, None)?;                                
                                let plan::RecordConc {mut fields} = lp.get_record_type(data_type.fields).clone();
                                let scalar_t = lp.scalar_types.insert(plan::ConcRef::Conc(plan::ScalarTypeConc::Record(dt)));
                                
                                fields.insert(named.clone(), scalar_t);
        
                                let new_type = plan::Data { fields: lp.record_types.insert(plan::ConcRef::Conc(
                                    plan::RecordConc { fields }
                                )), stream: data_type.stream } ;
    
                                Ok(
                                    LinearBuilderState { 
                                        data_out: new_type, 
                                        op_kind: plan::OperatorKind::Access { access_after: *mo, op: plan::AccessOperator::DeRef { 
                                            input: prev_edge, reference, access , named, table: table_id_copy, output: next_edge } }, 
                                        call_span: call.span(), 
                                        update_mo: true
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
