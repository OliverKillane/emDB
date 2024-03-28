macro_rules! choices {
    (otherwise => $q:expr) => {$q};
    ($p:expr => $q:expr , $($ts:tt)+) => {
        choice($p, $q, choices!($($ts)+))
    };
}

macro_rules! create_operator {
    ($op:ident : $($m:ident :: $t:ident),*) => {

        $(
            mod $m;
            use $m::$t;
        )*

        #[derive(Debug)]
        pub(crate) enum $op {
            $(
                $t($t),
            )*
        }

        pub fn parse_operator() -> impl TokenParser<$op> {
            choices! {
                $(
                    peekident($t::NAME) => mapsuc($t::build_parser(), $op::$t),
                )*
                otherwise => error(gettoken, |t| Diagnostic::spanned(t.span(), Level::Error, format!("expected an operator but got {}", t)))
            }
        }

        pub fn build_logical(
            op: $op,
            lp: &mut LogicalPlan,
            tn: &HashMap<Ident, TableKey>,
            qk: QueryKey,
            vs: &mut HashMap<Ident, VarState>,
            cont: Option<Continue>,
        ) -> Result<StreamContext, LinkedList<Diagnostic>> {
            match op {
                $(
                    $op::$t(i) => i.build_logical(lp, tn, qk, vs, cont),
                )*
            }
        }
    };
}

create_operator!(Operator: op_fold::Fold, op_return::Return);