use super::*;

#[derive(Debug)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct Sort {
    call: Ident,
    fields: Vec<(Ident, (SortOrder, Span))>,
}

impl EMQLOperator for Sort {
    const NAME: &'static str = "sort";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                listseptrailing(
                    ',',
                    mapsuc(
                        seq(
                            setrepr(getident(), "<field>"),
                            setrepr(choices!(
                                peekident("asc") => mapsuc(matchident("asc"), |t| (SortOrder::Asc, t.span())),
                                peekident("desc") => mapsuc(matchident("desc"), |t| (SortOrder::Desc, t.span())),
                                // TODO: replace with call to errors::
                                otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("Can only sort by `asc` or `desc`, not by {t:?}")))
                            ), "<asc/desc>")
                        ),
                        |(i, (o, s))| (i, (o, s)),
                    ),
                ),
            ),
            |(call, fields)| Sort { call, fields },
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
        
        // check it is a stream
        // check the fields exist
        // check the sort does not conflict (one use)

        if let Some(cont) = cont {
            linear_builder(
                lp, 
                op_ctx, 
                cont, 
                |lp, op_ctx, prev, next_edge| {
                    let rec_type = lp.get_record_type(prev.data_type.fields);
                    let (raw_fields, mut errors) = extract_fields(fields, errors::sort_field_used_twice);
                    let mut sort_order = Vec::new();
                    for (field, (ordering, _)) in raw_fields {
                        let rec_field = field.clone().into();
                        if rec_type.fields.contains_key(&rec_field) {
                            sort_order.push((rec_field, convert_ordering(ordering)));
                        } else {
                            errors.push_back(errors::query_reference_field_missing(&field));
                        }
                    }

                    if !prev.data_type.stream {
                        errors.push_back(errors::query_stream_single_connection(call.span(), prev.last_span, true))
                    }

                    if errors.is_empty() {
                        Ok(
                            LinearBuilderState { 
                                data_out: prev.data_type, 
                                op: (plan::Sort { input: prev.prev_edge, sort_order, output: next_edge }.into()), 
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


fn convert_ordering(ordering: SortOrder) -> plan::SortOrder {
    match ordering {
        SortOrder::Asc => plan::SortOrder::Asc,
        SortOrder::Desc => plan::SortOrder::Desc,
    }
}