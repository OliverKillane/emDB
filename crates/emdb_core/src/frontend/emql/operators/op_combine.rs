use super::*;

#[derive(Debug)]
pub struct Combine {
    call: Ident,
    left_name: Ident,
    right_name: Ident,
    fields: Vec<(Ident, Expr)>,
}

impl EMQLOperator for Combine {
    const NAME: &'static str = "combine";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                seqs!(
                    matchident("use"),
                    setrepr(getident(), "<left input>"),
                    matchpunct('+'),
                    setrepr(getident(), "<right input>"),
                    matchident("in"),
                    listseptrailing(
                        ',',
                        mapsuc(
                            seqs!(
                                setrepr(getident(), "<field o combine to>"),
                                matchpunct('='),
                                setrepr(syntopunct(peekpunct(',')), "<combined value>")
                            ),
                            |(field, (_, expr))| (field, expr)
                        )
                    )
                ),
            ),
            |(call, (_, (left_name, (_, (right_name, (_, fields))))))| Combine {
                call,
                left_name,
                right_name,
                fields,
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
            left_name,
            right_name,
            fields,
        } = self;

        if let Some(cont) = cont {
            linear_builder(lp, op_ctx, cont, |lp, ctx, prev, next_edge| {
                let (raw_fields, mut errors) =
                    extract_fields_ordered(fields, errors::query_operator_field_redefined);

                if !prev.data_type.stream {
                    errors.push_back(errors::query_stream_single_connection(
                        call.span(),
                        prev.last_span,
                        true,
                    ))
                }

                let FieldComparison {
                    extra_fields,
                    missing_fields,
                } = check_fields_type(
                    lp,
                    prev.data_type.fields,
                    raw_fields.iter().map(|(id, _)| id),
                );

                for field in extra_fields {
                    errors.push_back(errors::query_combine_extra_field(
                        lp,
                        &call,
                        field,
                        &prev.data_type.fields,
                    ));
                }

                for field in missing_fields {
                    errors.push_back(errors::query_combine_missing_field(
                        lp,
                        &call,
                        field,
                        &prev.data_type.fields,
                    ));
                }

                if errors.is_empty() {
                    Ok(LinearBuilderState {
                        data_out: plan::Data {
                            fields: prev.data_type.fields,
                            stream: false,
                        },
                        op: plan::Combine {
                            input: prev.prev_edge,
                            left_name,
                            right_name,
                            update_fields: raw_fields
                                .into_iter()
                                .map(|(id, expr)| (id.into(), expr))
                                .collect(),
                            output: next_edge,
                        }
                        .into(),
                        call_span: call.span(),
                    })
                } else {
                    Err(errors)
                }
            })
        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
