use crate::plan::RecordConc;

use super::*;

#[derive(Debug)]
enum MatchKind {
    Equi{left_field: Ident, right_field: Ident},
    Pred(Expr),
    Cross
}

#[derive(Debug)]
enum JoinKind {
    Inner,
    Outer,
    Left,
}

#[derive(Debug)]
pub struct Join {
    call: Ident,
    left: Ident,
    right: Ident,
    matcher: MatchKind,
    join_kind: JoinKind
}

impl EMQLOperator for Join {
    const NAME: &'static str = "join";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        fn kind_parser() -> impl TokenParser<JoinKind> {
            choices!(
                peekident("inner") => mapsuc(matchident("inner"), |_| JoinKind::Inner),
                peekident("outer") => mapsuc(matchident("outer"), |_| JoinKind::Outer),
                peekident("left") => mapsuc(matchident("left"), |_| JoinKind::Left),
                otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("expected join kind `inner`, `outer` or `left` but got `{}`", t)))
            )
        }

        fn match_parser() -> impl TokenParser<MatchKind> {
            choices!(
                peekident("equi") => mapsuc(seq(
                    matchident("equi"),
                    recovgroup(Delimiter::Parenthesis, seqs!(
                        getident(),
                        matchpunct('='),
                        getident()
                    ))
                ), |(_, (l, (_, r)))| MatchKind::Equi{left_field: l, right_field: r}),
                peekident("pred") => mapsuc(seq(
                    matchident("pred"),
                    recovgroup(Delimiter::Brace, syn(collectuntil(isempty())))
                    ), |(_, e)| MatchKind::Pred(e)
                ),
                peekident("cross") => mapsuc(matchident("cross"), |_| MatchKind::Cross),
                otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("expected match kind `equi`, `pred` or `cross` but got `{}`", t)))
            )
        }
        
        mapsuc(
            functional_style(Self::NAME, seqs!(
                    matchident("use"),
                    getident(),
                    recovgroup(Delimiter::Bracket, seq(kind_parser(), match_parser())),
                    matchident("use"),
                    getident()
                )
            ),
            |(call, (_, (left, ((join_kind, matcher), (_, right)))))| Join {
                call,
                left,
                right,
                matcher,
                join_kind
            }
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
        fn get_input(id: &Ident, vs: &mut HashMap<Ident, VarState>, errors: &mut LinkedList<Diagnostic>) -> Option<Continue> {
            match vs.get(id) {
                Some(VarState::Available { created, state }) => 
                    {
                        let state = state.clone();
                        *vs.get_mut(id).unwrap() = VarState::Used { created: *created, used: id.span() };
                        Some(state)
                    }
                ,
                Some(VarState::Used { created, used }) => 
                    {errors.push_back(errors::query_use_variable_already_used(id, *created, *used)); None},
                
    
                None => {errors.push_back(errors::query_invalid_variable_use(id, vs)); None},
            }
        }

        fn check_field(lp: &plan::Plan, call: &Ident, field: &Ident, rec: &RecordConc, errors: &mut LinkedList<Diagnostic>) {
            let field_rec = field.clone().into();
            if !rec.fields.contains_key(&field_rec) {
                errors.push_back(errors::access_field_missing(call, field, get_user_fields(rec)));
            }
        }
        

        let Self { call, left, right, matcher, join_kind } = self;
        if cont.is_none() {
            let mut errors = LinkedList::new();
            
            let left_input = get_input(&left, vs, &mut errors);
            let right_input = get_input(&right, vs, &mut errors);

            if let (Some(left_cont), Some(right_cont)) = (left_input, right_input) {
                let left_rec_conc = lp.get_record_type_conc(left_cont.data_type.fields);
                let right_rec_conc = lp.get_record_type_conc(right_cont.data_type.fields);
                
                if !left_cont.data_type.stream {
                    errors.push_back(errors::operator_requires_streams(&call, &left));
                }

                if !right_cont.data_type.stream {
                    errors.push_back(errors::operator_requires_streams(&call, &right));
                }

                let op_matcher = match matcher {
                    MatchKind::Equi { left_field, right_field } => {
                        check_field(lp, &call, &left_field, left_rec_conc, &mut errors);
                        check_field(lp, &call, &right_field, left_rec_conc, &mut errors);

                        plan::MatchKind::Equi { left_field: left_field.into(), right_field: right_field.into() }
                    },
                    MatchKind::Pred(e) => plan::MatchKind::Pred(e),
                    MatchKind::Cross => plan::MatchKind::Cross,
                };

                // TODO: expose plan types to the user?
                let join_kind = match join_kind {
                    JoinKind::Inner => plan::JoinKind::Inner,
                    JoinKind::Outer => plan::JoinKind::Outer,
                    JoinKind::Left => plan::JoinKind::Left,
                };

                if errors.is_empty() {
                    let next_edge = lp.dataflow.insert(plan::DataFlow::Null);
                    let join_op = lp.operators.insert(plan::Join {
                        left: left_cont.prev_edge,
                        right: right_cont.prev_edge,
                        match_kind: op_matcher,
                        join_kind,
                        output: next_edge
                    }.into());
    
    
                    update_incomplete(lp.get_mut_dataflow(left_cont.prev_edge), join_op);
                    update_incomplete(lp.get_mut_dataflow(right_cont.prev_edge), join_op);

                    let left_scalar= lp.scalar_types.insert(plan::ConcRef::Conc(plan::ScalarTypeConc::Record(left_cont.data_type.fields)));
                    let right_scalar= lp.scalar_types.insert(plan::ConcRef::Conc(plan::ScalarTypeConc::Record(right_cont.data_type.fields)));
                    
                    let join_record = lp.record_types.insert(plan::ConcRef::Conc(plan::RecordConc { fields: HashMap::from([
                        (left.into(), left_scalar),
                        (right.into(), right_scalar),
                    ]) }));

                    let join_data = plan::Data { fields: join_record, stream: true };
                    *lp.get_mut_dataflow(next_edge) = plan::DataFlow::Incomplete { from: join_op, with: join_data.clone() };
                    lp.get_mut_context(op_ctx).add_operator(join_op);

                    Ok(StreamContext::Continue(Continue { data_type: join_data, prev_edge: next_edge, last_span: call.span() }))
                } else {
                    Err(errors)
                }
            } else {
                Err(errors)
            }
        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}