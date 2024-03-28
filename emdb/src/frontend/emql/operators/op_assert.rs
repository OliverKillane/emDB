use super::*;

#[derive(Debug)]
pub struct Assert {
    call: Ident,
    expr: Expr,
}

impl EMQLOperator for Assert {
    const NAME: &'static str = "assert";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, syn(collectuntil(isempty()))),
            |(call, expr)| Assert { call, expr },
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
        let Self { call, expr } = self;
        Err(singlelist(errors::operator_unimplemented(&call)))
    }
}
