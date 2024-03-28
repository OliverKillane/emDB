use super::*;

#[derive(Debug)]
pub struct Fold {
    call: Ident,
    initial: Vec<(Ident, (AstType, Expr))>,
    update: Vec<(Ident, Expr)>,
}

impl EMQLOperator for Fold {
    const NAME: &'static str = "fold";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                seqs!(
                    recovgroup(Delimiter::Parenthesis, fields_assign()),
                    matchpunct('='),
                    matchpunct('>'),
                    recovgroup(Delimiter::Parenthesis, fields_expr())
                ),
            ),
            |(call, (initial, (_, (_, update))))| Fold {
                call,
                initial,
                update,
            },
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
        let Self {
            call,
            initial,
            update,
        } = self;
        Err(singlelist(errors::operator_unimplemented(&call)))
    }
}
