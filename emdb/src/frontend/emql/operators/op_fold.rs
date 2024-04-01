use combi::tokens::{basic::peekpunct, derived::syntopunct};

use crate::frontend::emql::parse::{type_parser, type_parser_to_punct};

use super::*;

#[derive(Debug)]
pub struct Fold {
    call: Ident,
    fields: Vec<(Ident, (AstType, Expr, Expr))>
}

impl EMQLOperator for Fold {
    const NAME: &'static str = "fold";

    fn build_parser() -> impl TokenParser<Self> {
        mapsuc(
            functional_style(
                Self::NAME,
                listseptrailing(',',
    mapsuc(
            seqs!(
                getident(),
                matchpunct(':'),
                type_parser_to_punct('='),
                matchpunct('='),
                syntopunct(peekpunct('-')),
                matchpunct('-'),
                matchpunct('>'),
                syntopunct(peekpunct(','))
            ),
            |(id, (_, (t, (_, (initial, (_, (_, update)))))))| (id, (t, initial, update))
        )
    ),
            ),
            |(call, fields)| Fold {
                call,
                fields
            },
        )
    }

    fn build_logical(
        self,
        lp: &mut plan::LogicalPlan,
        tn: &HashMap<Ident, plan::Key<plan::Table>>,
        qk: plan::Key<plan::Query>,
        vs: &mut HashMap<Ident, VarState>,
        ts: &mut HashMap<Ident, plan::Key<plan::ScalarType>>,
        mo: &mut Option<plan::Key<plan::Operator>>,
        cont: Option<Continue>,
    ) -> Result<StreamContext, LinkedList<Diagnostic>> {
        let Self {
            call,
            fields
        } = self;
        if let Some(Continue { data_type, prev_edge, last_span }) = cont {
            // must input stream
            // create output type
            // bing chilling



          Err(singlelist(errors::operator_unimplemented(&call)))


        } else {
            Err(singlelist(errors::query_cannot_start_with_operator(&call)))
        }
    }
}
