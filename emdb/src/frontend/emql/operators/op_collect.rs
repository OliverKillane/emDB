//! Collect a stream into a single data structure
//! - EMDB can determine the type of the collection
//! - Use can reference the collection type using their provided type alias
use super::*;

#[derive(Debug)]
pub struct Collect {
    call: Ident,
    field: Ident,
    alias: Ident,
}

impl EMQLOperator for Collect {
    const NAME: &'static str = "collect";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, seqs!(
                setrepr(getident(), "<field to collect to>"),
                matchident("as"),
                matchident("type"),
                setrepr(getident(), "<type alias for the collection>")
            )),
            |(call, (field, (_, (_, alias))))| Collect{call, field, alias}
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
        let Self {call, field, alias } = self;
        if let Some(cont) = cont {
            linear_builder(
                lp,
                qk,
                mo,
                cont,
                |lp, mo, Continue { data_type, prev_edge, last_span }, next_edge| {
                    if let Some((orig, _)) = ts.get_key_value(&alias) {
                        Err(singlelist(errors::collect_type_alias_redefined(&alias, orig)))
                    } else {
                        let bag_type = lp.scalar_types.insert(plan::ConcRef::Conc(plan::ScalarTypeConc::Bag(data_type.fields)));
                        ts.insert(alias, bag_type);
                        let record_type = lp.record_types.insert(plan::ConcRef::Conc(plan::RecordConc{ fields: HashMap::from([(field.clone(), bag_type)]) } )); 
                        Ok(
                            LinearBuilderState { 
                                data_out: plan::Data {
                                    fields: record_type,
                                    stream: false,
                                }, op_kind: plan::OperatorKind::Pure(
                                    plan::PureOperator::Collect { input: prev_edge, into: field, output: next_edge }
                                ), call_span: call.span(), update_mo: false }
                        )
                    }
                }
            )
        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
