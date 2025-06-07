use std::num::NonZero;

use super::ir::*;
use smart_arenas::prelude::*;



impl<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N: Namer>
    Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N>
{
    pub fn new(
        token_msgs: Token<'msgs>,
        token_seqs: Token<'seqs>,
        token_stages: Token<'stages>,
        token_items: Token<'items>,
        token_bools: Token<'bools>,
        token_ints: Token<'ints>,
        token_consts: Token<'consts>,
        namer: N,
    ) -> Self {
        Self {
            msgs: Share::new(0, token_msgs),
            seqs: Own::new(0, token_seqs),
            stages: Own::new(0, token_stages),
            items: App::new(0, token_items),
            bools: App::new(0, token_bools),
            ints: App::new(0, token_ints),
            consts: Share::new(0, token_consts),
            namer,
        }
    }
}

impl<N: Namer, Data> Assigned<N, Data> {
    pub fn get_name_and_span(&self) -> (N::Name, N::Span) {
        (self.ident.clone().into(), self.ident.clone().into())
    }
}

impl <P: PrimitiveAssoc> Primitive<P> {
    pub fn size(&self) -> NonZero<usize> { // TODO(oliverkillane): switch to some 'bits type' instead of usize
        match self {
            Primitive::Bit(_) => NonZero::new(1).unwrap(), // TODO(oliverkillane): remove unwraps in a nicer way
            Primitive::Byte(_) => NonZero::new(8).unwrap(),
            Primitive::Integer { underlying, assoc: _ } => underlying.bits.into(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    
    // JUSTIFY: Deriving Eq on empty struct
    //  - Allows us to use namer in tests
    #[derive(PartialEq, Eq, Debug)]
    pub struct TestNamer;

    #[derive(PartialEq, Eq, Hash, Clone, Debug)]
    pub struct TestSpan(pub usize);

    #[derive(PartialEq, Eq, Hash, Clone, Debug)]
    pub struct TestName(pub &'static str);

    #[derive(PartialEq, Eq, Hash, Clone)]
    pub struct TestIdent {
        pub name: TestName,
        pub span: TestSpan,
    }

    impl From<TestIdent> for TestSpan {
        fn from(value: TestIdent) -> Self {
            value.span
        }
    }

    impl From<TestIdent> for TestName {
        fn from(value: TestIdent) -> Self {
            value.name
        }
    }

    impl Namer for TestNamer {
        type Span = TestSpan;
        type Name = TestName;
        type Ident = TestIdent;
    }

    pub fn plan_test_impl<F>(test: F)
    where
        F: for<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts> FnOnce(
            Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, TestNamer>,
        ),
    {
        multiple_context!(
            token_msgs,
            token_seqs,
            token_stages,
            token_items,
            token_bools,
            token_ints,
            token_consts => {
                test(Plan::new(
                    token_msgs, token_seqs, token_stages, token_items, token_bools, token_ints, token_consts, TestNamer));
            }
        );
    }

    // JUSTIFY: Helper for plan construction.
    //  - Just using the `multiple_context!` macro directly means the test code is not formatted 
    //    (is part of macro input)
    #[macro_export]
    macro_rules! plan_tests {
        ($module:ident => $($name:ident,)*) => {
            mod $module {
                $(
                    #[test]
                    fn $name() {
                        crate::plan::helpers::tests::plan_test_impl(super::$name);
                    }
                )*
            }
        };
    }
}
