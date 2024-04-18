use super::*;

#[derive(Debug)]
pub struct Ref {
    call: Ident,
    table_name: Ident,
    out_as: Ident,
}

impl EMQLOperator for Ref {
    const NAME: &'static str = "ref";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(seqs!(
            matchident("ref"), 
            getident(),
            matchident("as"),
            getident()
        ), |(call, (table_name, (_, out_ref)))| {
            Ref { call, table_name, out_as: out_ref }
        })
    }

    fn build_logical(
        self,
        lp: &mut plan::Plan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        op_ctx: plan::Key<plan::Context>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, table_name, out_as: out_ref } = self;
        if cont.is_none() {
            if let Some(table_id) = tn.get(&table_name) {
                Ok(StreamContext::Continue(create_scanref(lp, op_ctx, *table_id, out_ref.into(), call.span())))
            } else {
                Err(singlelist(errors::query_nonexistent_table(
                    &call,
                    &table_name,
                )))
            }
        } else {
            Err(singlelist(errors::query_operator_cannot_come_first(&call)))
        }
    }
}
