use crate::utils::conster::{conster, Const};

pub(crate) trait TokenSource {
    type Token;
    fn next(&mut self) -> Option<Self::Token>;
}

type Parser<I: TokenSource, O, E> = fn(I) -> (I, Result<O, E>);
type Recov<I: TokenSource> = fn(I) -> I;

enum ThenFail<O1, E1, E2> {
    First { err: E1 },
    Second { first: O1, err: E2 },
}

#[inline(always)]
const fn then<
    I: TokenSource,
    O1,
    E1,
    P1: Const<Parser<I, O1, E1>>,
    O2,
    E2,
    P2: Const<Parser<I, O2, E2>>,
>() -> Parser<I, (O1, O2), ThenFail<O1, E1, E2>> {
    |ts| match P1::val()(ts) {
        (ts, Ok(o1)) => match P2::val()(ts) {
            (ts, Ok(o2)) => (ts, Ok((o1, o2))),
            (ts, Err(e2)) => (ts, Err(ThenFail::Second { first: o1, err: e2 })),
        },
        (ts, Err(e1)) => (ts, Err(ThenFail::First { err: e1 })),
    }
}

const fn a(x: i32, y: i32) -> (i32, i32) {
    (x + y, x)
}

#[inline(always)]
fn terminal<I: TokenSource>(mut input: I) -> (I, Result<(), ()>) {
    while input.next().is_some() {}
    (input, Ok(()))
}

const fn Backtrack() {}

#[cfg(test)]
mod test {
    use super::*;

    struct DummyTokens {
        toks: Vec<i32>,
    }

    impl TokenSource for DummyTokens {
        type Token = i32;
        fn next(&mut self) -> Option<Self::Token> {
            self.toks.pop()
        }
    }

    fn dummy_parse(mut input: DummyTokens) -> (DummyTokens, Result<i32, ()>) {
        match input.next() {
            Some(i) => (input, Ok(i)),
            None => (input, Err(())),
        }
    }

    conster! {
        const Terminal: Parser<DummyTokens, (), ()> = terminal;
        const DummyI32: Parser<DummyTokens, i32, ()> = dummy_parse;
        const Then = then::<DummyTokens, i32, (), DummyI32, (), (), Terminal>
    }

    #[test]
    fn compile_terminal() {}
}
