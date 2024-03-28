use super::*;

#[derive(Debug)]
pub struct Collect {
    call: Ident,
}

impl EMQLOperator for Collect {
    const NAME: &'static str = "collect";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(getident(), |call| Collect { call })
    }

    fn build_logical(
        self,
        lp: &mut LogicalPlan,
        tn: &HashMap<Ident, TableKey>,
        qk: QueryKey,
        vs: &mut HashMap<Ident, VarState>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call } = self;
        Err(singlelist(errors::operator_unimplemented(&call)))
    }
}
