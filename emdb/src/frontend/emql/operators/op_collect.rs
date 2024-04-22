//! Collect a stream into a single data structure
//! - EMDB can determine the type of the collection
//! - Use can reference the collection type using their provided type alias
use super::*;

#[derive(Debug)]
pub struct Collect {
    call: Ident,
    field: Ident,
    alias: Option<Ident>,
}

impl EMQLOperator for Collect {
    const NAME: &'static str = "collect";

    fn build_parser(ctx_recur: ContextRecurHandle) -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, seqs!(
                setrepr(getident(), "<field to collect to>"),
                choice(
                    isempty(), 
                    mapsuc(nothing(), |()| None), 
                    mapsuc(seqs!(
                        matchident("as"),
                        matchident("type"),
                        setrepr(getident(), "<type alias for the collection>")
                    ), |(_, (_, x))| Some(x))
                )
            )),
            |(call, (field, alias))| Collect{call, field, alias}
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
        let Self {call, field, alias } = self;
        if let Some(cont) = cont {
            linear_builder(
                lp,
                op_ctx,
                cont,
                |lp, mo, Continue { data_type, prev_edge, last_span }, next_edge| {
                    let bag_type = lp.scalar_types.insert(plan::ConcRef::Conc(plan::ScalarTypeConc::Bag(data_type.fields)));
                    
                    if let Some(alias_id) = alias {
                        if let Some((orig, _)) = ts.get_key_value(&alias_id) {
                            return Err(singlelist(errors::collect_type_alias_redefined(&alias_id, orig)))
                        } else {
                            ts.insert(alias_id, bag_type);
                        }
                    }

                    let record_type = lp.record_types.insert(
                        plan::ConcRef::Conc(plan::RecordConc{ fields: HashMap::from([(field.clone().into(), bag_type)]) } )); 
                    Ok(
                        LinearBuilderState { 
                            data_out: plan::Data {
                                fields: record_type,
                                stream: false,
                            }, 
                            op: plan::Collect { input: prev_edge, into: field.into(), output: next_edge }.into(), 
                            call_span: call.span()
                        }
                    )
                }
            )
        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
