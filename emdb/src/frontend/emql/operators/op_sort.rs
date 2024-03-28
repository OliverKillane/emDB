use super::*;

#[derive(Debug)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct Sort {
    call: Ident,
    fields: Vec<(Ident, (SortOrder, Span))>,
}

impl EMQLOperator for Sort {
    const NAME: &'static str = "sort";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                listseptrailing(
                    ',',
                    mapsuc(
                        seq(
                            getident(),
                            choices!(
                                peekident("asc") => mapsuc(matchident("asc"), |t| (SortOrder::Asc, t.span())),
                                peekident("desc") => mapsuc(matchident("desc"), |t| (SortOrder::Desc, t.span())),
                                // TODO remplace with call to errors::
                                otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("Can only sort by `asc` or `desc`, not by {:?}", t)))
                            ),
                        ),
                        |(i, (o, s))| (i, (o, s)),
                    ),
                ),
            ),
            |(call, fields)| Sort { call, fields },
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