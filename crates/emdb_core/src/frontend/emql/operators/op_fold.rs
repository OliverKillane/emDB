//! Apply a fold over a stream of values to create a single value.
//! 
//! Semantics:
//! ```text
//! <stream> |> fold(<unique field>: <type> = <initial> -> <update>, ... ) ~> <single>
//! ```
use super::*;
#[derive(Debug)]
pub struct Fold {
    call: Ident,
    fields: Vec<(Ident, (AstType, Expr, Expr))>
}

impl EMQLOperator for Fold {
    const NAME: &'static str = "fold";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                listseptrailing(',',
                mapsuc(
                        seqs!(
                            setrepr(getident(), "<field>"),
                            matchpunct(':'),
                            type_parser_to_punct('='),
                            matchpunct('='),
                            setrepr(syntopunct(peekpunct('-')), "<initial value>"),
                            matchpunct('-'),
                            matchpunct('>'),
                            setrepr(syntopunct(peekpunct(',')), "<update value>")
                        ),
                        |(id, (_, (t, (_, (initial, (_, (_, update)))))))| (id, (t, initial, update))
                    )
                ),
            ),
            |(call, fields)| Fold {
                call,
                fields
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
            fields
        } = self;
        if let Some(cont) = cont {
            linear_builder(lp,  op_ctx, cont, 
            |lp, mo, prev, next_edge| {
                    let (raw_fields, mut errors) = extract_fields_ordered(fields, errors::query_operator_field_redefined);
                    if !prev.data_type.stream {
                        errors.push_back(errors::query_stream_single_connection(call.span(), prev.last_span, true))
                    }

                    let mut type_fields = HashMap::new();
                    let mut fold_fields = Vec::new();

                    for (field, (typ, initial, update)) in raw_fields {
                        if let Some(scalar_t) = result_to_opt(query_ast_typeto_scalar(tn, ts, typ, |e| errors::query_nonexistent_table(&call, e), errors::query_no_cust_type_found), &mut errors) {
                            let data_type = lp.scalar_types.insert(scalar_t);
                            type_fields.insert(field.clone().into(), data_type);
                            fold_fields.push((field.into(), plan::FoldField { initial, update }));                        
                        }
                    }

                    if errors.is_empty() {
                        Ok(
                            LinearBuilderState { 
                                data_out: plan::Data { 
                                    fields: lp.record_types.insert(plan::ConcRef::Conc(plan::RecordConc {fields: type_fields})), 
                                    stream: false 
                                },
                                op: plan::Fold { input: prev.prev_edge, fold_fields, output: next_edge }.into(), 
                                call_span: call.span()}
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
