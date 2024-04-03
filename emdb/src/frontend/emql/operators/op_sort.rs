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
                            setrepr(getident(), "<field>"),
                            setrepr(choices!(
                                peekident("asc") => mapsuc(matchident("asc"), |t| (SortOrder::Asc, t.span())),
                                peekident("desc") => mapsuc(matchident("desc"), |t| (SortOrder::Desc, t.span())),
                                // TODO remplace with call to errors::
                                otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("Can only sort by `asc` or `desc`, not by {t:?}")))
                            ), "<asc/desc>")
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
        lp: &mut plan::Plan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        mo: &mut Option<plan::Key<plan::Operator>>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self { call, fields } = self;
        Err(singlelist(errors::operator_unimplemented(&call)))
    }
}
