use std::marker::PhantomData;

pub(crate) trait TokenSource {
    type Token;
    fn next(&mut self) -> Option<Self::Token>;
}

pub(crate) trait Parser<I: TokenSource> {
    type Output;
    type Error;
    fn parse(self, input: I) -> (I, Result<Self::Output, Self::Error>);
}

pub(crate) trait Recov<E, I: TokenSource> {
    fn recover(self, input: I, err: E) -> (I, E);
}

pub(crate) struct Then<I: TokenSource, P1: Parser<I>, P2: Parser<I>> {
    first: P1,
    second: P2,
    _marker: PhantomData<I>,
}

impl<I: TokenSource, P1: Parser<I>, P2: Parser<I>> Then<I, P1, P2> {
    pub(crate) fn new(first: P1, second: P2) -> Self {
        Self {
            first,
            second,
            _marker: PhantomData,
        }
    }
}

pub(crate) enum ThenFail<O1, E1, E2> {
    First { err: E1 },
    Second { first: O1, err: E2 },
}

impl<I: TokenSource, P1: Parser<I>, P2: Parser<I>> Parser<I> for Then<I, P1, P2> {
    type Output = (P1::Output, P2::Output);
    type Error = ThenFail<P1::Output, P1::Error, P2::Error>;

    fn parse(self, input: I) -> (I, Result<Self::Output, Self::Error>) {
        let (input, o1) = self.first.parse(input);
        match o1 {
            Ok(o1) => {
                let (input, o2) = self.second.parse(input);
                match o2 {
                    Ok(o2) => (input, Ok((o1, o2))),
                    Err(e2) => (input, Err(ThenFail::Second { first: o1, err: e2 })),
                }
            }
            Err(e1) => (input, Err(ThenFail::First { err: e1 })),
        }
    }
}

struct Terminal<I: TokenSource> {
    _marker: PhantomData<I>,
}

impl <I: TokenSource> Terminal<I> {
    pub(crate) fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<I: TokenSource> Parser<I> for Terminal<I> {
    type Output = ();
    type Error = ();

    fn parse(self, mut input: I) -> (I, Result<Self::Output, Self::Error>) {
        while input.next().is_some() {}
        (input, Ok(()))
    }
}

struct Recover<I: TokenSource, P: Parser<I>, R: Recov<P::Error, I>> {
    parser: P,
    recover: R,
    _marker: PhantomData<I>,
} 

impl <I: TokenSource, P: Parser<I>, R: Recov<P::Error, I>> Recover<I, P, R> {
    pub(crate) fn new(parser: P, recover: R) -> Self {
        Self {
            parser,
            recover,
            _marker: PhantomData,
        }
    }
}

impl <I: TokenSource, P: Parser<I>, R: Recov<P::Error, I>> Parser<I> for Recover<I, P, R> {
    type Output = P::Output;
    type Error = P::Error;

    fn parse(self, input: I) -> (I, Result<Self::Output, Self::Error>) {
        let (input, o) = self.parser.parse(input);
        match o {
            Ok(o) => (input, Ok(o)),
            Err(e) => {
                let (ts, e) = self.recover.recover(input, e);
                (ts, Err(e))
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test(){

        Then::new(, )


    }
}
