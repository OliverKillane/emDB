use super::*;

#[derive(Debug)]
pub struct Row {
    call: Ident,
    fields: Vec<(Ident, (AstType, Expr))>,
}

impl EMQLOperator for Row {
    const NAME: &'static str = "row";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, fields_assign()),
            |(call, fields)| Row { call, fields },
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
        let Self { call, fields } = self;
        Err(singlelist(errors::operator_unimplemented(&call)))
    }
}
