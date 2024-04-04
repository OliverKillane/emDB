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

    fn build_parser() -> impl TokenParser<Self> {
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
                                // TODO: remplace with call to errors::
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
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        mo: &mut Option<plan::Key<plan::Operator>>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, fields } = self;
        
        // check it is a stream
        // check the fields exist
        // check the sort does not conflict (one use)

        if let Some(cont) = cont {
            linear_builder(
                lp, 
                qk, 
                mo, 
                cont, 
                |lp, mo, prev, next_edge| {
                    let rec_type = lp.get_record_type(prev.data_type.fields);
                    let (raw_fields, mut errors) = extract_fields(fields, errors::sort_field_used_twice);
                    let mut sort_order = Vec::new();
                    for (field, (ordering, _)) in raw_fields {
                        if rec_type.fields.contains_key(&field) {
                            sort_order.push((field, convert_ordering(ordering)));
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
                                data_out: prev.data_type, op_kind: plan::OperatorKind::Pure(plan::PureOperator::Sort { input: prev.prev_edge, sort_order, output: next_edge }), call_span: call.span(), update_mo: false }
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