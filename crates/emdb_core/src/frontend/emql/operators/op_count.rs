use super::*;

#[derive(Debug)]
pub struct Count {
    call: Ident,
    out_field: Ident,
}

impl EMQLOperator for Count {
    const NAME: &'static str = "count";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, getident()),
            |(call, out_field)| Self { call, out_field },
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
        let Self { call, out_field } = self;
        let x: usize = 0;
        if let Some(cont) = cont {
            linear_builder(lp, op_ctx, cont, |lp, ctx, prev, next_edge| {
                if !prev.data_type.stream {
                    Err(singlelist(errors::query_stream_single_connection(
                        call.span(),
                        prev.last_span,
                        true,
                    )))
                } else {
                    // NOTE: This operator needs to produce a usize type.
                    //       - It is not truly a user type, but it is a rust primitive, so we assume it is available
                    //       - It 'kinda' is breaking the separation between implementation and frontend, as we rely 
                    //         on the count operator giving us a usize. Ordinarily we ensure only user code (expressions) 
                    //         are susceptible to changes in the backend's types
                    // The span for the out_field is used to ensure type errors propagate to user code, rather than 
                    // `Span::call_site()`. 
                    
                    let usize_span = out_field.span();
                    let out_field = plan::RecordField::User(out_field);

                    let size_type =
                        lp.scalar_types
                            .insert(plan::ConcRef::Conc(plan::ScalarTypeConc::Rust {
                                type_context: plan::TypeContext::Query,
                                ty: syn::parse2(quote::quote_spanned!(usize_span => usize)).unwrap(),
                            }));
                    
                    Ok(LinearBuilderState {
                        data_out: plan::Data {
                            fields: lp
                                .record_types
                                .insert(plan::ConcRef::Conc(plan::RecordConc {
                                    fields: HashMap::from([(out_field.clone(), size_type)]),
                                })),
                            stream: false,
                        },
                        op: plan::Count {
                            input: prev.prev_edge,
                            output: next_edge,
                            out_field,
                        }
                        .into(),
                        call_span: call.span(),
                    })
                }
            })
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
