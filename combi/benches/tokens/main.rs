//! Benchmarks for the combi parser, compared with others.
//! - each case is in [`cases`], and defines a parse that will pass.
//! - no parsers should fail, and the failure is ignored (TODO: need to be caseful about this with microbenchmarks & optimisation)
//!
//! | Method | Reason for inclusion |
//! | Combi | is this library |
//! | Hand-Rolled | How most token parsing for macros is done |
//! | Chumsky-Proc | A bridge to parse tokenstreams using chumsky, which is a parser combinator library supporting recovery |
//!
use proc_macro2::TokenStream;
use std::fmt::Debug;

mod cases;
use cases::{LongSequence, Nothing, RecursiveIdent};
mod parsers;
use parsers::{chumsky_proc::ChumskyProc, combi::CombiParser, handrolled::HandRolled};

trait Parseable: Eq + PartialEq + Debug {
    type Param;
    fn generate_case(param: Self::Param) -> Self;
    fn generate_tokens(&self) -> TokenStream;
}

trait Parse<O> {
    fn parse(input: TokenStream) -> O;
}

macro_rules! impl_cases {
    ($($case:ident as $name:ident for [ $($arg:tt)* ] ),* ) => {
        $(
            #[divan::bench(
                name = stringify!($name),
                types = [CombiParser, HandRolled, ChumskyProc],
                args = [ $($arg)* ]
            )]
            fn $name<P: Parse<$case>>(bencher: divan::Bencher, param: <$case as Parseable>::Param) {
                let o = $case::generate_case(param);
                let tks = o.generate_tokens();
                assert_eq!(P::parse(tks.clone()), o);
                bencher.bench_local(|| {
                    P::parse(tks.clone())
                })
            }
        )*
    }
}

impl_cases! {
    RecursiveIdent as recursive_ident for [1,2,64],
    LongSequence as long_sequence for [0, 100, 1000000],
    Nothing as parse_nothing for [()] // redundant here, const param ignored
}

fn main() {
    divan::main();
}
