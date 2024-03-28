use super::*;

#[derive(Debug)]
pub struct Map {
    call: Ident,
    new_fields: Vec<(Ident, (AstType, Expr))>,
}

impl EMQLOperator for Map {
    const NAME: &'static str = "map";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(Self::NAME, fields_assign()),
            |(call, new_fields)| Map { call, new_fields },
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
        let Self { call, new_fields } = self;
        Err(singlelist(errors::operator_unimplemented(&call)))
    }
}
