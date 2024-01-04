//! The core parser combinators required to work on any kind of input.
//! - No constraints on the input type, fully generalised.

// TODO: need to clean up constraints so that all match Seq (constraints on function, versus struct, versus impl)

use std::marker::PhantomData;
use std::rc::{Rc, Weak};

/// A trait for combining continuations and successed into single continuations
/// - A continuation may contain many previous syntax errors, upon the next parser
///   succeeding, rather than propagate it's success, we want to propagate the previous
///   errors (with potentially some information from the success)
pub trait ConComb<O, C> {
    fn combine_out(self, out: O) -> Self;
    fn combine_con(self, con: C) -> Self;
}

/// A trait for combining a continutation and an error
/// - Used when continuations are passed into a parser, which irrecoverably fails
///   to combine the continuation's information in, and propagate.
pub trait ErrComb<C> {
    fn combine_con(self, con: C) -> Self;
}

pub enum ParseResult<E, C, O> {
    /// Parse has failed, throws error information.
    Err(E),

    /// Parse failed, but we can still continue with some context
    Con(C),

    /// Parse Successful
    Suc(O),
}

/// The core [Parser] trait for [crate]
pub trait Parser<I> {
    /// The success output type
    type O;

    /// If the parser (or some child parser) could not get success, but could
    /// recover, and wants to propagate this information
    type C;

    /// The parser failed with this error.
    type E;

    #[allow(clippy::type_complexity)]
    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>);
}

/// A trait for consumers that recover from an error, and can manipulate that error.
/// (e.g. to add the span of tokens skipped due to failure)
/// If recovery fails, it can simply add its own error to the continuation.
pub trait Recover<I, E> {
    type C;
    fn recover(&self, input: I, err: E) -> (I, Self::C);
}

pub struct ID<I, P1: Parser<I>> {
    parser: P1,
    _marker: PhantomData<I>,
}

/// A parser that does nothing
pub fn id<I, P: Parser<I>>(parser: P) -> ID<I, P> {
    ID {
        parser,
        _marker: PhantomData,
    }
}

impl<I, P: Parser<I>> Parser<I> for ID<I, P> {
    type O = P::O;
    type C = P::C;
    type E = P::E;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        self.parser.parse(input)
    }
}

pub struct Seq<I, E, C, P1: Parser<I, E = E, C = C>, P2: Parser<I, E = E, C = C>>
where
    P2::C: ConComb<P2::O, P1::C>,
    P2::E: ErrComb<P2::C>,
{
    parser1: P1,
    parser2: P2,
    _marker: PhantomData<I>,
}

/// Combines two parsers
/// ```text
/// seq(
///   O1, O2 => (O1, O2)
///   O1, C2  => C
///   O1, E  => E
///   C1 , O2 => C2(C1, O2)
///   E , _  => E
/// )
/// ```
/// - The error type of `p1` must be convertible into the second
/// - Continuations are propagated
pub fn seq<I, E, C, P1: Parser<I, E = E, C = C>, P2: Parser<I, E = E, C = C>>(
    p1: P1,
    p2: P2,
) -> Seq<I, E, C, P1, P2>
where
    P2::C: ConComb<P2::O, P1::C>,
    P2::E: ErrComb<P2::C>,
{
    Seq {
        parser1: p1,
        parser2: p2,
        _marker: PhantomData,
    }
}

impl<I, E, C, P1: Parser<I, E = E, C = C>, P2: Parser<I, E = E, C = C>> Parser<I>
    for Seq<I, E, C, P1, P2>
where
    P2::C: ConComb<P2::O, P1::C>,
    P2::E: ErrComb<P2::C>,
{
    type O = (P1::O, P2::O);
    type C = P2::C;
    type E = P2::E;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        let (next_input, res) = self.parser1.parse(input);
        match res {
            ParseResult::Err(e) => (next_input, ParseResult::Err(e)),
            ParseResult::Con(c) => {
                let (final_input, next_res) = self.parser2.parse(next_input);
                (
                    final_input,
                    match next_res {
                        ParseResult::Err(e) => ParseResult::Err(e.combine_con(c)),
                        ParseResult::Con(c2) => ParseResult::Con(c2.combine_con(c)),
                        ParseResult::Suc(o) => ParseResult::Con(c.combine_out(o)),
                    },
                )
            }
            ParseResult::Suc(o) => {
                let (final_input, next_res) = self.parser2.parse(next_input);
                (
                    final_input,
                    match next_res {
                        ParseResult::Err(e) => ParseResult::Err(e),
                        ParseResult::Con(c2) => ParseResult::Con(c2),
                        ParseResult::Suc(o2) => ParseResult::Suc((o, o2)),
                    },
                )
            }
        }
    }
}

pub struct Recov<I, P: Parser<I>, R: Recover<I, P::E, C = P::C>> {
    parser: P,
    recov: R,
    _marker: PhantomData<I>,
}

/// Recovers `parser` failure through `recov`
pub fn recover<I, P: Parser<I>, R: Recover<I, P::E, C = P::C>>(
    parser: P,
    recov: R,
) -> Recov<I, P, R> {
    Recov {
        parser,
        recov,
        _marker: PhantomData,
    }
}

impl<I, P: Parser<I>, R: Recover<I, P::E, C = P::C>> Parser<I> for Recov<I, P, R> {
    type O = P::O;
    type C = P::C;
    type E = P::E;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        let (next_input, res) = self.parser.parse(input);
        match res {
            ParseResult::Err(e) => {
                let (final_input, c) = self.recov.recover(next_input, e);
                (final_input, ParseResult::Con(c))
            }
            ParseResult::Con(c) => (next_input, ParseResult::Con(c)),
            ParseResult::Suc(o) => (next_input, ParseResult::Suc(o)),
        }
    }
}

pub struct Either<
    I,
    EP,
    CP,
    OP,
    P1: Parser<I, E = EP, C = CP, O = OP>,
    P2: Parser<I, E = EP, C = CP, O = OP>,
    CH: Parser<I, E = EP, C = CP, O = bool>,
> {
    parser1: P1,
    parser2: P2,
    choice: CH,
    _marker: PhantomData<I>,
}

/// Takes two parsers, and a choice parser, and uses the choice parser to decide which parser to apply.
/// - the error and continuation for all parsers must be the same.
pub fn either<
    I,
    EP,
    CP,
    OP,
    P1: Parser<I, E = EP, C = CP, O = OP>,
    P2: Parser<I, E = EP, C = CP, O = OP>,
    CH: Parser<I, E = EP, C = CP, O = bool>,
>(
    choice: CH,
    parser1: P1,
    parser2: P2,
) -> Either<I, EP, CP, OP, P1, P2, CH> {
    Either {
        parser1,
        parser2,
        choice,
        _marker: PhantomData,
    }
}

impl<
        I,
        EP,
        CP,
        OP,
        P1: Parser<I, E = EP, C = CP, O = OP>,
        P2: Parser<I, E = EP, C = CP, O = OP>,
        CH: Parser<I, E = EP, C = CP, O = bool>,
    > Parser<I> for Either<I, EP, CP, OP, P1, P2, CH>
{
    type O = OP;
    type C = CP;
    type E = EP;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        let (next_input, res) = self.choice.parse(input);
        match res {
            ParseResult::Err(e) => (next_input, ParseResult::Err(e)),
            ParseResult::Con(c) => (next_input, ParseResult::Con(c)),
            ParseResult::Suc(b) => {
                if b {
                    self.parser1.parse(next_input)
                } else {
                    self.parser2.parse(next_input)
                }
            }
        }
    }
}

pub struct Many1<I, EP, CP, S: Parser<I, E = EP, C = CP, O = bool>, P: Parser<I, E = EP, C = CP>>
where
    CP: ConComb<P::O, CP>,
    EP: ErrComb<CP>,
{
    sep: S,
    parser: P,
    _marker: PhantomData<I>,
}

/// Parses the pattern (P S P S ... S P S?) with S determining if the parsing should continue until the separator parser ends it.
/// - the separator returns a boolean for if parsing should stop, it consumes the input (internally decides if it backtracks, consumes)
pub fn many1<I, EP, CP, S: Parser<I, E = EP, C = CP, O = bool>, P: Parser<I, E = EP, C = CP>>(
    sep: S,
    parser: P,
) -> Many1<I, EP, CP, S, P>
where
    CP: ConComb<P::O, CP>,
    EP: ErrComb<CP>,
{
    Many1 {
        sep,
        parser,
        _marker: PhantomData,
    }
}

impl<I, EP, CP, S: Parser<I, E = EP, C = CP, O = bool>, P: Parser<I, E = EP, C = CP>> Parser<I>
    for Many1<I, EP, CP, S, P>
where
    CP: ConComb<P::O, CP>,
    EP: ErrComb<CP>,
{
    type O = Vec<P::O>;
    type C = CP;
    type E = EP;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        let mut curr_input = input;
        let mut outs: Vec<P::O> = vec![];
        let mut con = None;

        let (next_input, res) = self.parser.parse(curr_input);

        match res {
            ParseResult::Err(e) => return (next_input, ParseResult::Err(e)),
            ParseResult::Con(c) => con = Some(c),
            ParseResult::Suc(o) => outs.push(o),
        }

        curr_input = next_input;

        loop {
            let (next_input, sep_res) = self.sep.parse(curr_input);
            match sep_res {
                ParseResult::Err(e) => {
                    return (
                        next_input,
                        ParseResult::Err(if let Some(c) = con {
                            e.combine_con(c)
                        } else {
                            e
                        }),
                    )
                }
                ParseResult::Con(c) => {
                    if let Some(c2) = con {
                        con = Some(c2.combine_con(c));
                    } else {
                        con = Some(c);
                    }
                }
                ParseResult::Suc(b) => {
                    if !b {
                        return (
                            next_input,
                            if let Some(c) = con {
                                ParseResult::Con(c)
                            } else {
                                ParseResult::Suc(outs)
                            },
                        );
                    }
                }
            }

            let (next_next_input, parse_res) = self.parser.parse(next_input);
            match parse_res {
                ParseResult::Err(e) => {
                    return (
                        next_next_input,
                        ParseResult::Err(if let Some(c) = con {
                            e.combine_con(c)
                        } else {
                            e
                        }),
                    )
                }
                ParseResult::Con(c) => {
                    if let Some(c2) = con {
                        con = Some(c2.combine_con(c));
                    } else {
                        con = Some(c);
                    }
                }
                ParseResult::Suc(o) => {
                    if let Some(c) = con {
                        con = Some(c.combine_out(o));
                    } else {
                        outs.push(o);
                    }
                }
            }
            curr_input = next_next_input;
        }
    }
}

pub struct Many0<I, EP, CP, S: Parser<I, E = EP, C = CP, O = bool>, P: Parser<I, E = EP, C = CP>>
where
    CP: ConComb<P::O, CP>,
    EP: ErrComb<CP>,
{
    sep: S,
    parser: P,
    _marker: PhantomData<I>,
}

/// Parses the pattern (S P ... S P S?) with S determining if the parsing should continue.
/// - the separator returns a boolean for if parsing should stop, it consumes the input (internally decides if it backtracks, consumes)
pub fn many0<I, EP, CP, S: Parser<I, E = EP, C = CP, O = bool>, P: Parser<I, E = EP, C = CP>>(
    sep: S,
    parser: P,
) -> Many0<I, EP, CP, S, P>
where
    CP: ConComb<P::O, CP>,
    EP: ErrComb<CP>,
{
    Many0 {
        sep,
        parser,
        _marker: PhantomData,
    }
}

// TODO: use common function form Many1 and Many0

impl<I, EP, CP, S: Parser<I, E = EP, C = CP, O = bool>, P: Parser<I, E = EP, C = CP>> Parser<I>
    for Many0<I, EP, CP, S, P>
where
    CP: ConComb<P::O, CP>,
    EP: ErrComb<CP>,
{
    type O = Vec<P::O>;
    type C = CP;
    type E = EP;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        let mut curr_input = input;
        let mut res = vec![];
        let mut con = None;

        loop {
            let (next_input, sep_res) = self.sep.parse(curr_input);
            match sep_res {
                ParseResult::Err(e) => {
                    return (
                        next_input,
                        ParseResult::Err(if let Some(c) = con {
                            e.combine_con(c)
                        } else {
                            e
                        }),
                    )
                }
                ParseResult::Con(c) => {
                    if let Some(c2) = con {
                        con = Some(c2.combine_con(c));
                    } else {
                        con = Some(c);
                    }
                }
                ParseResult::Suc(b) => {
                    if !b {
                        return (
                            next_input,
                            if let Some(c) = con {
                                ParseResult::Con(c)
                            } else {
                                ParseResult::Suc(res)
                            },
                        );
                    }
                }
            }

            let (next_next_input, parse_res) = self.parser.parse(next_input);
            match parse_res {
                ParseResult::Err(e) => {
                    return (
                        next_next_input,
                        ParseResult::Err(if let Some(c) = con {
                            e.combine_con(c)
                        } else {
                            e
                        }),
                    )
                }
                ParseResult::Con(c) => {
                    if let Some(c2) = con {
                        con = Some(c2.combine_con(c));
                    } else {
                        con = Some(c);
                    }
                }
                ParseResult::Suc(o) => {
                    if let Some(c) = con {
                        con = Some(c.combine_out(o));
                    } else {
                        res.push(o);
                    }
                }
            }
            curr_input = next_next_input;
        }
    }
}

pub struct MapSuc<I, P: Parser<I>, O, F: Fn(P::O) -> O> {
    parser: P,
    func: F,
    _marker: PhantomData<I>,
}

/// Map the result of a parser on success
pub fn mapsuc<I, P: Parser<I>, O, F: Fn(P::O) -> O>(parser: P, func: F) -> MapSuc<I, P, O, F> {
    MapSuc {
        parser,
        func,
        _marker: PhantomData,
    }
}

impl<I, P: Parser<I>, MO, F: Fn(P::O) -> MO> Parser<I> for MapSuc<I, P, MO, F> {
    type O = MO;
    type C = P::C;
    type E = P::E;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        let (next_input, res) = self.parser.parse(input);
        (
            next_input,
            match res {
                ParseResult::Suc(o) => ParseResult::Suc((self.func)(o)),
                ParseResult::Con(c) => ParseResult::Con(c),
                ParseResult::Err(e) => ParseResult::Err(e),
            },
        )
    }
}

pub struct MapErr<I, P: Parser<I>, E, F: Fn(P::E) -> E> {
    parser: P,
    func: F,
    _marker: PhantomData<I>,
}

/// Map the result of a parser on success
pub fn maperr<I, P: Parser<I>, E, F: Fn(P::E) -> E>(parser: P, func: F) -> MapErr<I, P, E, F> {
    MapErr {
        parser,
        func,
        _marker: PhantomData,
    }
}

impl<I, P: Parser<I>, ME, F: Fn(P::E) -> ME> Parser<I> for MapErr<I, P, ME, F> {
    type O = P::O;
    type C = P::C;
    type E = ME;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        let (next_input, res) = self.parser.parse(input);
        (
            next_input,
            match res {
                ParseResult::Suc(o) => ParseResult::Suc(o),
                ParseResult::Con(c) => ParseResult::Con(c),
                ParseResult::Err(e) => ParseResult::Err((self.func)(e)),
            },
        )
    }
}

pub struct MapAll<I, O, P: Parser<I>, F: Fn(P::O) -> ParseResult<P::E, P::C, O>> {
    parser: P,
    func: F,
    _marker: (PhantomData<I>, PhantomData<O>),
}
pub fn mapall<I, O, P: Parser<I>, F: Fn(P::O) -> ParseResult<P::E, P::C, O>>(
    parser: P,
    func: F,
) -> MapAll<I, O, P, F> {
    MapAll {
        parser,
        func,
        _marker: (PhantomData, PhantomData),
    }
}
impl<I, O, P: Parser<I>, F: Fn(P::O) -> ParseResult<P::E, P::C, O>> Parser<I>
    for MapAll<I, O, P, F>
{
    type O = O;
    type C = P::C;
    type E = P::E;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        let (next_input, res) = self.parser.parse(input);
        (
            next_input,
            match res {
                ParseResult::Suc(o) => (self.func)(o),
                ParseResult::Con(c) => ParseResult::Con(c),
                ParseResult::Err(e) => ParseResult::Err(e),
            },
        )
    }
}

pub struct LiftInput<I1, I2, P: Parser<I2>> {
    parser: P,
    to: fn(I1) -> I2,
    from: fn(I2) -> I1,
    _marker: PhantomData<I1>,
}

/// Lifts into a diferent input type.
/// - Useful when you need to use a different type of parser (e.g from non-backtracking to backtracking)
pub fn lift<I1, I2, P: Parser<I2>>(
    parser: P,
    to: fn(I1) -> I2,
    from: fn(I2) -> I1,
) -> LiftInput<I1, I2, P> {
    LiftInput {
        parser,
        to,
        from,
        _marker: PhantomData,
    }
}

impl<I1, I2, P: Parser<I2>> Parser<I1> for LiftInput<I1, I2, P> {
    type O = P::O;
    type C = P::C;
    type E = P::E;

    fn parse(&self, input: I1) -> (I1, ParseResult<Self::E, Self::C, Self::O>) {
        let (next_input, res) = self.parser.parse((self.to)(input));
        ((self.from)(next_input), res)
    }
}

pub struct Recursive<I, O, E, C> {
    parser: Rc<Box<dyn Parser<I, O = O, E = E, C = C>>>,
    _marker: PhantomData<I>,
}

pub struct RecursiveHandle<I, O, E, C> {
    parser: Weak<Box<dyn Parser<I, O = O, E = E, C = C>>>, // using dyn to avoid an infinite type
    _marker: PhantomData<I>,
}

// TODO: object safety prevents
impl<I, O, E, C> Clone for RecursiveHandle<I, O, E, C> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            _marker: PhantomData,
        }
    }
}
impl<I, O, E, C> Parser<I> for RecursiveHandle<I, O, E, C> {
    type C = C;
    type E = E;
    type O = O;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        // NOTE: Recursive handler only used within the context of a recursive
        //       parser, so we can assume on parse the item is present.
        self.parser.upgrade().unwrap().parse(input)
    }
}

pub fn recursive<I, O, E, C, P: Parser<I, O = O, E = E, C = C> + 'static>(
    parse: fn(RecursiveHandle<I, O, E, C>) -> P,
) -> Recursive<I, O, E, C> {
    Recursive {
        parser: Rc::new_cyclic(move |w| {
            Box::new((parse)(RecursiveHandle {
                parser: w.clone(),
                _marker: PhantomData,
            }))
        }),
        _marker: PhantomData,
    }
}

impl<I, O, E, C> Parser<I> for Recursive<I, O, E, C> {
    type C = C;
    type E = E;
    type O = O;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        self.parser.parse(input)
    }
}

pub struct Or<I, E, C, P1: Parser<I, O = bool, C = C, E = E>, P2: Parser<I, O = bool, C = C, E = E>>
{
    p1: P1,
    p2: P2,
    _marker: (PhantomData<I>, PhantomData<E>, PhantomData<C>),
}

pub fn or<I, E, C, P1: Parser<I, O = bool, C = C, E = E>, P2: Parser<I, O = bool, C = C, E = E>>(
    p1: P1,
    p2: P2,
) -> Or<I, E, C, P1, P2> {
    Or {
        p1,
        p2,
        _marker: (PhantomData, PhantomData, PhantomData),
    }
}
impl<I, E, C, P1: Parser<I, O = bool, C = C, E = E>, P2: Parser<I, O = bool, C = C, E = E>>
    Parser<I> for Or<I, E, C, P1, P2>
{
    type O = bool;
    type C = C;
    type E = E;

    fn parse(&self, input: I) -> (I, ParseResult<Self::E, Self::C, Self::O>) {
        let (res_input, res) = self.p1.parse(input);
        match res {
            ParseResult::Err(e) => (res_input, ParseResult::Err(e)),
            ParseResult::Con(c) => (res_input, ParseResult::Con(c)),
            ParseResult::Suc(b) => {
                if b {
                    (res_input, ParseResult::Suc(true))
                } else {
                    let (res_input, res) = self.p2.parse(res_input);
                    match res {
                        ParseResult::Err(e) => (res_input, ParseResult::Err(e)),
                        ParseResult::Con(c) => (res_input, ParseResult::Con(c)),
                        ParseResult::Suc(b) => (res_input, ParseResult::Suc(b)),
                    }
                }
            }
        }
    }
}