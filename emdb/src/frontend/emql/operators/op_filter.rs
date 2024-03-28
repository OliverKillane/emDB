use super::*;

#[derive(Debug)]
pub struct Filter {
    call: Ident,
    filter_expr: Expr,
}

impl EMQLOperator for Filter {
    const NAME: &'static str = "filter";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, syn(collectuntil(isempty()))),
            |(call, filter_expr)| Filter { call, filter_expr },
        )
    }

    fn build_logical(
        self,
        lp: &mut LogicalPlan,
        tn: &HashMap<Ident, TableKey>,
        qk: QueryKey,
        vs: &mut HashMap<Ident, VarState>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, filter_expr } = self;
        Err(singlelist(errors::operator_unimplemented(&call)))
    }
}
