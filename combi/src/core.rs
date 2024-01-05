//! The core, use agnostic, combinators for [Combi] upon which others can be constructed.
//! - Each has a type, a construction function and a [Combi] implementation.

use super::*;
use derive_where::derive_where;
use std::fmt::Debug;
use std::{
    marker::PhantomData,
    rc::{Rc, Weak},
};

/// Applies a [Combi] with no changes
/// ```
/// # use combi::{Combi, CombiResult, core::{id, nothing}};
/// matches!(id(id(id(nothing::<(), (), i32>()))).comp(3), (3, CombiResult::Suc(())));
/// ```
#[allow(non_camel_case_types)]
#[derive_where(Clone; P: Clone)]
#[derive_where(Debug; P: Debug)]
pub struct id<P: Combi>(pub P);
impl<P: Combi> Combi for id<P> {
    type Suc = P::Suc;
    type Err = P::Err;
    type Con = P::Con;
    type Inp = P::Inp;
    type Out = P::Out;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        self.0.comp(input)
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.repr(f)
    }
}

/// Applies a [Combi] with no changes
pub fn nothing<E, C, I>() -> NothingP<E, C, I> {
    NothingP(PhantomData, PhantomData, PhantomData)
}

#[derive_where(Clone, Debug)]
pub struct NothingP<E, C, I>(PhantomData<E>, PhantomData<C>, PhantomData<I>);

impl<E, C, I> Combi for NothingP<E, C, I> {
    type Suc = ();
    type Err = E;
    type Con = C;
    type Inp = I;
    type Out = I;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        (input, CombiResult::Suc(()))
    }

    fn repr(&self, _: &mut Formatter<'_>) -> Result<(), Error> {
        Ok(())
    }
}

/// Applies a first [Combi], then attempts the second if it is [successful](Combi::Suc) or a [continuation](Combi::Con), returns both results as a tuple.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct seq<P1, P2>(pub P1, pub P2)
where
    P1: Combi,
    P2: Combi<Con = P1::Con, Err = P1::Err, Inp = P1::Out, Out = P1::Out>,
    P1::Con: CombiCon<P2::Suc, P1::Con>,
    P1::Err: CombiErr<P1::Con>;

impl<P1, P2> Combi for seq<P1, P2>
where
    P1: Combi,
    P2: Combi<Con = P1::Con, Err = P1::Err, Inp = P1::Out, Out = P1::Out>,
    P1::Con: CombiCon<P2::Suc, P1::Con>,
    P1::Err: CombiErr<P1::Con>,
{
    type Suc = (P1::Suc, P2::Suc);
    type Err = P1::Err;
    type Con = P1::Con;
    type Inp = P1::Inp;
    type Out = P2::Out;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (p1_out, p1_res) = self.0.comp(input);
        match p1_res {
            CombiResult::Suc(s) => {
                let (p2_out, p2_res) = self.1.comp(p1_out);
                (
                    p2_out,
                    match p2_res {
                        CombiResult::Suc(s2) => CombiResult::Suc((s, s2)),
                        CombiResult::Con(c) => CombiResult::Con(c),
                        CombiResult::Err(e) => CombiResult::Err(e),
                    },
                )
            }
            CombiResult::Con(c) => {
                let (p2_out, p2_res) = self.1.comp(p1_out);
                (
                    p2_out,
                    match p2_res {
                        CombiResult::Suc(s2) => CombiResult::Con(c.combine_suc(s2)),
                        CombiResult::Con(c2) => CombiResult::Con(c.combine_con(c2)),
                        CombiResult::Err(e) => CombiResult::Err(e.inherit_con(c)),
                    },
                )
            }
            CombiResult::Err(e) => (p1_out, CombiResult::Err(e)),
        }
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}{}", Repr(&self.0), Repr(&self.1))
    }
}

pub enum DiffRes<F, S> {
    First(F),
    Second(S),
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct seqdiff<P1, P2>(pub P1, pub P2)
where
    P1: Combi,
    P2: Combi<Inp = P1::Out, Err = P1::Err, Con = P1::Con>,
    P1::Err: CombiErr<P1::Con>,
    P1::Con: CombiCon<P2::Suc, P1::Con>;

impl<P1, P2> Combi for seqdiff<P1, P2>
where
    P1: Combi,
    P2: Combi<Inp = P1::Out, Err = P1::Err, Con = P1::Con>,
    P1::Err: CombiErr<P1::Con>,
    P1::Con: CombiCon<P2::Suc, P1::Con>,
{
    type Suc = (P1::Suc, P2::Suc);
    type Err = P1::Err;
    type Con = P1::Con;
    type Inp = P1::Inp;
    type Out = DiffRes<P1::Out, P2::Out>;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (p1_out, p1_res) = self.0.comp(input);
        match p1_res {
            CombiResult::Suc(s) => {
                let (p2_out, p2_res) = self.1.comp(p1_out);
                (
                    DiffRes::Second(p2_out),
                    match p2_res {
                        CombiResult::Suc(s2) => CombiResult::Suc((s, s2)),
                        CombiResult::Con(c) => CombiResult::Con(c),
                        CombiResult::Err(e) => CombiResult::Err(e),
                    },
                )
            }
            CombiResult::Con(c) => {
                let (p2_out, p2_res) = self.1.comp(p1_out);
                (
                    DiffRes::Second(p2_out),
                    match p2_res {
                        CombiResult::Suc(s2) => CombiResult::Con(c.combine_suc(s2)),
                        CombiResult::Con(c2) => CombiResult::Con(c.combine_con(c2)),
                        CombiResult::Err(e) => CombiResult::Err(e.inherit_con(c)),
                    },
                )
            }
            CombiResult::Err(e) => (DiffRes::First(p1_out), CombiResult::Err(e)),
        }
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}{}", Repr(&self.0), Repr(&self.1))
    }
}

/// Applies a provided function to [successful](Combi::Suc) results.
#[allow(non_camel_case_types)]
#[derive_where(Clone; P: Clone, F: Clone)]
#[derive_where(Debug; P: Debug, F: Debug)]
pub struct mapsuc<S, P, F>(pub P, pub F)
where
    F: Fn(P::Suc) -> S,
    P: Combi;

impl<S, P, F> Combi for mapsuc<S, P, F>
where
    F: Fn(P::Suc) -> S,
    P: Combi,
{
    type Suc = S;
    type Err = P::Err;
    type Con = P::Con;
    type Inp = P::Inp;
    type Out = P::Out;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (p_out, p_res) = self.0.comp(input);
        (
            p_out,
            match p_res {
                CombiResult::Suc(s) => CombiResult::Suc((self.1)(s)),
                CombiResult::Con(c) => CombiResult::Con(c),
                CombiResult::Err(e) => CombiResult::Err(e),
            },
        )
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.repr(f)
    }
}

/// Applies a provided function to [erroneous](Combi::Err) results.
#[allow(non_camel_case_types)]
#[derive_where(Clone; P: Clone, F: Clone)]
#[derive_where(Debug; P: Debug, F: Debug)]
pub struct maperr<E, P, F>(pub P, pub F)
where
    F: Fn(P::Err) -> E,
    P: Combi;

impl<E, P, F> Combi for maperr<E, P, F>
where
    F: Fn(P::Err) -> E,
    P: Combi,
{
    type Suc = P::Suc;
    type Err = E;
    type Con = P::Con;
    type Inp = P::Inp;
    type Out = P::Out;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (p_out, p_res) = self.0.comp(input);
        (
            p_out,
            match p_res {
                CombiResult::Suc(s) => CombiResult::Suc(s),
                CombiResult::Con(c) => CombiResult::Con(c),
                CombiResult::Err(e) => CombiResult::Err((self.1)(e)),
            },
        )
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.repr(f)
    }
}

/// Applies the primary parser, if it [succeeds](Combi::Suc) or is a [continuation](Combi::Con) then this is returned.
///
/// If it errors, then the recovery parser is provided the error and output as its input.
/// The recovery parser can [error](Combi::Err) (could not recover), [continue](Combi::Con) (it has recovered to allow further combis to continue), or [succeed](Combi::Suc) (entirely recovered from failure with parsed result)
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct recover<P, R>(pub P, pub R)
where
    P: Combi,
    R: Combi<Inp = (P::Err, P::Out), Out = P::Out, Con = P::Con>;

impl<P, R> Combi for recover<P, R>
where
    P: Combi,
    R: Combi<Inp = (P::Err, P::Out), Out = P::Out, Con = P::Con, Suc = P::Suc>,
{
    type Suc = P::Suc;
    type Err = R::Err;
    type Con = P::Con;
    type Inp = P::Inp;
    type Out = P::Out;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (p_out, p_res) = self.0.comp(input);
        match p_res {
            CombiResult::Suc(s) => (p_out, CombiResult::Suc(s)),
            CombiResult::Con(c) => (p_out, CombiResult::Con(c)),
            CombiResult::Err(e) => {
                let (r_out, r_res) = self.1.comp((e, p_out));
                (r_out, r_res)
            }
        }
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.repr(f)
    }
}

/// Based on a [Combi] producing a `bool`, determine the next [Combi] to compute.
/// - if the choice is a [continuation](Combi::Con), then it is converted to an error.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct choice<CP, PT, PF>(pub CP, pub PT, pub PF)
where
    CP: Combi<Suc = bool>,
    PT: Combi<Inp = CP::Out, Out = CP::Out, Err = CP::Err>,
    PF: Combi<Inp = CP::Out, Out = CP::Out, Suc = PT::Suc, Con = PT::Con, Err = CP::Err>,
    PT::Err: CombiErr<CP::Con>;

impl<CP, PT, PF> Combi for choice<CP, PT, PF>
where
    CP: Combi<Suc = bool>,
    PT: Combi<Inp = CP::Out, Out = CP::Out, Err = CP::Err>,
    PF: Combi<Inp = CP::Out, Out = CP::Out, Suc = PT::Suc, Con = PT::Con, Err = CP::Err>,
    PT::Err: CombiErr<CP::Con>,
{
    type Suc = PT::Suc;
    type Err = CP::Err;
    type Con = PT::Con;
    type Inp = CP::Inp;
    type Out = PT::Out;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (cp_out, cp_res) = self.0.comp(input);
        match cp_res {
            CombiResult::Suc(s) => {
                if s {
                    self.1.comp(cp_out)
                } else {
                    self.2.comp(cp_out)
                }
            }
            CombiResult::Con(c) => (cp_out, CombiResult::Err(CombiErr::catch_con(c))),
            CombiResult::Err(e) => (cp_out, CombiResult::Err(e)),
        }
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{} => ({} | {})",
            Repr(&self.0),
            Repr(&self.1),
            Repr(&self.2)
        )
    }
}

#[allow(non_camel_case_types)]
#[derive_where(Clone; CP: Clone, PS: Clone, PN: Clone)]
#[derive_where(Debug; CP: Debug, PS: Debug, PN: Debug)]
pub struct choicesome<S, CP, PS, PN>(pub CP, pub PS, pub PN)
where
    CP: Combi<Suc = Option<S>>,
    PS: Combi<Inp = (S, CP::Out), Out = CP::Out, Err = CP::Err>,
    PN: Combi<Inp = CP::Out, Out = CP::Out, Con = PS::Con, Err = CP::Err, Suc = PS::Suc>,
    PS::Err: CombiErr<CP::Con>;

impl<S, CP, PS, PN> Combi for choicesome<S, CP, PS, PN>
where
    CP: Combi<Suc = Option<S>>,
    PS: Combi<Inp = (S, CP::Out), Out = CP::Out, Err = CP::Err>,
    PN: Combi<Inp = CP::Out, Out = CP::Out, Con = PS::Con, Err = CP::Err, Suc = PS::Suc>,
    PS::Err: CombiErr<CP::Con>,
{
    type Suc = PS::Suc;
    type Err = CP::Err;
    type Con = PS::Con;
    type Inp = CP::Inp;
    type Out = CP::Out;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (cp_out, cp_res) = self.0.comp(input);
        match cp_res {
            CombiResult::Suc(s) => {
                if let Some(s_inner) = s {
                    self.1.comp((s_inner, cp_out))
                } else {
                    self.2.comp(cp_out)
                }
            }
            CombiResult::Con(c) => (cp_out, CombiResult::Err(CombiErr::catch_con(c))),
            CombiResult::Err(e) => (cp_out, CombiResult::Err(e)),
        }
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{} => {} else {}",
            Repr(&self.0),
            Repr(&self.1),
            Repr(&self.2)
        )
    }
}

/// Using the result of a previous [Combi], use a function to determine which [Combi] to compute next.
#[allow(non_camel_case_types)]
#[derive_where(Clone; SP: Clone, PSS: Clone, F: Clone, R: Clone)]
#[derive_where(Debug; SP: Debug, PSS: Debug, F: Debug, R: Debug)]
pub struct select<S, C, SP, PSS, F, R, const N: usize>(pub SP, pub PSS, pub F, pub R)
where
    SP: Combi,
    F: Fn(
        &PSS,
        SP::Suc,
    ) -> &dyn Combi<Inp = SP::Out, Out = SP::Out, Suc = S, Err = SP::Err, Con = C>,
    R: Fn(&PSS) -> [&dyn Combi<Inp = SP::Out, Out = SP::Out, Suc = S, Err = SP::Err, Con = C>; N],
    SP::Err: CombiErr<SP::Con>;

impl<S, C, SP, PSS, F, R, const N: usize> Combi for select<S, C, SP, PSS, F, R, N>
where
    SP: Combi,
    F: Fn(
        &PSS,
        SP::Suc,
    ) -> &dyn Combi<Inp = SP::Out, Out = SP::Out, Suc = S, Err = SP::Err, Con = C>,
    R: Fn(&PSS) -> [&dyn Combi<Inp = SP::Out, Out = SP::Out, Suc = S, Err = SP::Err, Con = C>; N],
    SP::Err: CombiErr<SP::Con>,
{
    type Suc = S;
    type Err = SP::Err;
    type Con = C;
    type Inp = SP::Inp;
    type Out = SP::Out;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (sp_out, sp_res) = self.0.comp(input);
        match sp_res {
            CombiResult::Suc(s) => (self.2)(&self.1, s).comp(sp_out),
            CombiResult::Con(c) => (sp_out, CombiResult::Err(CombiErr::catch_con(c))),
            CombiResult::Err(e) => (sp_out, CombiResult::Err(e)),
        }
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} => (", Repr(&self.0))?;
        for i in (self.3)(&self.1) {
            write!(f, "{} | ", Repr(i))?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

/// Lift the input through two functions
pub fn lift<I, O, P, FI, FO>(p: P, func_in: FI, func_out: FO) -> Lift<I, O, P, FI, FO>
where
    P: Combi,
    FI: Fn(I) -> P::Inp,
    FO: Fn(P::Out) -> O,
{
    Lift {
        p,
        func_in,
        func_out,
        _marker: PhantomData,
    }
}

#[derive_where(Clone; P: Clone, FI: Clone, FO: Clone)]
#[derive_where(Debug; P: Debug, FI: Debug, FO: Debug)]
pub struct Lift<I, O, P, FI, FO>
where
    P: Combi,
    FI: Fn(I) -> P::Inp,
    FO: Fn(P::Out) -> O,
{
    p: P,
    func_in: FI,
    func_out: FO,
    _marker: PhantomData<I>,
}

impl<I, O, P, FI, FO> Combi for Lift<I, O, P, FI, FO>
where
    P: Combi,
    FI: Fn(I) -> P::Inp,
    FO: Fn(P::Out) -> O,
{
    type Suc = P::Suc;
    type Err = P::Err;
    type Con = P::Con;
    type Inp = I;
    type Out = O;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (p_out, p_res) = self.p.comp((self.func_in)(input));
        ((self.func_out)(p_out), p_res)
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.p.repr(f)
    }
}

/// Lift the input and output to a new type for [Combi] with different inputs/outputs, and carry some of the context.
pub fn liftcarry<I, O, TL, P, FI, FO>(
    p: P,
    func_in: FI,
    func_out: FO,
) -> LiftCarry<I, O, TL, P, FI, FO>
where
    P: Combi,
    FI: Fn(I) -> (P::Inp, TL),
    FO: Fn(P::Out, TL) -> O,
{
    LiftCarry {
        p,
        func_in,
        func_out,
        _marker: PhantomData,
    }
}

#[derive_where(Clone; P: Clone, FI: Clone, FO: Clone)]
#[derive_where(Debug; P: Debug, FI: Debug, FO: Debug)]
pub struct LiftCarry<I, O, TL, P, FI, FO>
where
    P: Combi,
    FI: Fn(I) -> (P::Inp, TL),
    FO: Fn(P::Out, TL) -> O,
{
    p: P,
    func_in: FI,
    func_out: FO,
    _marker: PhantomData<(I, O, TL)>,
}

impl<I, O, TL, P, FI, FO> Combi for LiftCarry<I, O, TL, P, FI, FO>
where
    P: Combi,
    FI: Fn(I) -> (P::Inp, TL),
    FO: Fn(P::Out, TL) -> O,
{
    type Suc = P::Suc;
    type Err = P::Err;
    type Con = P::Con;
    type Inp = I;
    type Out = O;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (p_inp, tl) = (self.func_in)(input);
        let (p_out, p_res) = self.p.comp(p_inp);
        ((self.func_out)(p_out, tl), p_res)
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.p.repr(f)
    }
}

/// The recursion combinator.
///
/// //TODO: fix
/// ```ignore
/// # use combi::{Combi, CombiResult, core::{Seq, Recursive, Nothing}};
///
/// // an infinite combi:
/// let (r, _): (i32, CombiResult<(),(),()>) = Recursive(|r| Seq(Nothing(), r.clone())).comp(3);
/// ```
pub fn recursive<I, O, S, E, C, P, F>(f: F) -> Recursive<I, O, S, E, C>
where
    F: Fn(RecursiveHandle<I, O, S, E, C>) -> P,
    P: Combi<Inp = I, Out = O, Suc = S, Err = E, Con = C> + 'static,
{
    Recursive {
        p: Rc::new_cyclic(move |w| Box::new((f)(RecursiveHandle { p: w.clone() }))),
    }
}

type RecurBox<I, O, S, E, C> = Box<dyn Combi<Inp = I, Out = O, Suc = S, Con = C, Err = E>>;

#[derive_where(Clone, Debug)]
pub struct RecursiveHandle<I, O, S, E, C> {
    #[allow(clippy::type_complexity)]
    p: Weak<RecurBox<I, O, S, E, C>>,
}

impl<I, O, S, E, C> Combi for RecursiveHandle<I, O, S, E, C> {
    type Suc = S;
    type Err = E;
    type Con = C;
    type Inp = I;
    type Out = O;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        // INV: owned by some recursive parser, ptr always upgradable as dropping that parser drops this.
        self.p.upgrade().unwrap().comp(input)
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "...")
    }
}

// No Debug implementation for recursive
#[derive_where(Clone)]
pub struct Recursive<I, O, S, E, C> {
    #[allow(clippy::type_complexity)]
    p: Rc<RecurBox<I, O, S, E, C>>,
}

impl<I, O, S, E, C> Combi for Recursive<I, O, S, E, C> {
    type Suc = S;
    type Err = E;
    type Con = C;
    type Inp = I;
    type Out = O;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        self.p.comp(input)
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", Repr(&**(self.p)))
    }
}

/// Allows a [Combi] to be repeatedly parsed based on another combinator in the pattern:
/// ```text
/// S I S I S I ...
/// ```
/// Each `I` appends its success to the vector passed in the input.
#[allow(non_camel_case_types)]
#[derive_where(Clone; SP: Clone, IP: Clone)]
#[derive_where(Debug; SP: Debug, IP: Debug)]
pub struct manyappsep<O, SP, IP>(pub SP, pub IP)
where
    SP: Combi<Inp = O, Out = O, Suc = bool>,
    // Note as either can continue or error, the outputs must be the same, and hence the inputs must also be
    IP: Combi<Inp = O, Out = O, Con = SP::Con, Err = SP::Err>,
    SP::Con: CombiCon<IP::Suc, SP::Con>,
    SP::Err: CombiErr<SP::Con>;

impl<O, SP, IP> Combi for manyappsep<O, SP, IP>
where
    SP: Combi<Inp = O, Out = O, Suc = bool>,
    IP: Combi<Inp = O, Out = O, Con = SP::Con, Err = SP::Err>,
    SP::Con: CombiCon<IP::Suc, SP::Con>,
    SP::Err: CombiErr<SP::Con>,
{
    type Suc = Vec<IP::Suc>;
    type Err = SP::Err;
    type Con = SP::Con;
    type Inp = (Vec<IP::Suc>, O);
    type Out = O;

    fn comp(
        &self,
        (mut v, mut input): Self::Inp,
    ) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        // NOTE: Ideally I would declare a type enum `ResStat { Cont(Self::Cont), Succ(Self::Suc)}`
        //       and use this instead of separate inferred `Option<Self::Cont>` and `v`, however we
        //       cannot yet refer to associated types inside functions

        let mut cont = None;

        loop {
            let (sp_out, sp_res) = self.0.comp(input);
            match sp_res {
                CombiResult::Suc(false) => {
                    if let Some(c) = cont {
                        return (sp_out, CombiResult::Con(c));
                    } else {
                        return (sp_out, CombiResult::Suc(v));
                    }
                }
                CombiResult::Suc(true) => {
                    let (ip_out, ip_res) = self.1.comp(sp_out);
                    input = ip_out;
                    match ip_res {
                        CombiResult::Suc(s) => {
                            if let Some(c_prev) = cont {
                                cont = Some(c_prev.combine_suc(s));
                            } else {
                                v.push(s);
                            }
                        }
                        CombiResult::Con(c) => {
                            if let Some(c_prev) = cont {
                                cont = Some(c_prev.combine_con(c));
                            } else {
                                cont = Some(c);
                            }
                        }
                        CombiResult::Err(_) => todo!(),
                    }
                }
                CombiResult::Con(c) => {
                    return (sp_out, CombiResult::Err(CombiErr::catch_con(c)));
                }
                CombiResult::Err(e) => {
                    return (sp_out, CombiResult::Err(e));
                }
            }
        }
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}{}...", Repr(&self.0), Repr(&self.1))
    }
}

/// Allows a [Combi] to be repeatedly parsed until it returns a None.
/// - if the inner [Combi] returns a [continuation](Combi::Con), then it will continue computing.
#[allow(non_camel_case_types)]
#[derive_where(Clone; P: Clone)]
#[derive_where(Debug; P: Debug)]
pub struct manyappsome<S, O, P>(pub P)
where
    P: Combi<Suc = Option<S>, Inp = O, Out = O>,
    P::Con: CombiCon<S, P::Con>,
    P::Err: CombiErr<P::Con>;

impl<S, O, P> Combi for manyappsome<S, O, P>
where
    P: Combi<Suc = Option<S>, Inp = O, Out = O>,
    P::Con: CombiCon<S, P::Con>,
    P::Err: CombiErr<P::Con>,
{
    type Suc = Vec<S>;
    type Err = P::Err;
    type Con = P::Con;
    type Inp = (Vec<S>, O);
    type Out = O;

    fn comp(
        &self,
        (mut v, mut input): Self::Inp,
    ) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        // NOTE: Same as in [ManyAppendSep]

        let mut cont = None;

        loop {
            let (sp_out, sp_res) = self.0.comp(input);
            input = sp_out;
            match sp_res {
                CombiResult::Suc(None) => {
                    if let Some(c_prev) = cont {
                        return (input, CombiResult::Con(c_prev));
                    } else {
                        return (input, CombiResult::Suc(v));
                    }
                }
                CombiResult::Suc(Some(s)) => {
                    if let Some(c_prev) = cont {
                        cont = Some(c_prev.combine_suc(s));
                    } else {
                        v.push(s);
                    }
                }
                CombiResult::Con(c) => {
                    if let Some(c_prev) = cont {
                        cont = Some(c_prev.combine_con(c));
                    } else {
                        cont = Some(c);
                    }
                }
                CombiResult::Err(e) => {
                    return (input, CombiResult::Err(e));
                }
            }
        }
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}...", Repr(&self.0))
    }
}

pub fn pipemap<I, O, E, C, F>(f: F) -> PipeMap<I, O, E, C, F>
where
    F: Fn(I) -> O,
{
    PipeMap {
        f,
        _marker: PhantomData,
    }
}
#[derive_where(Clone; F: Clone)]
#[derive_where(Debug; F: Debug)]
pub struct PipeMap<I, O, E, C, F>
where
    F: Fn(I) -> O,
{
    f: F,
    _marker: PhantomData<(I, O, E, C)>,
}

impl<I, O, E, C, F> Combi for PipeMap<I, O, E, C, F>
where
    F: Fn(I) -> O,
{
    type Suc = ();
    type Err = E;
    type Con = C;
    type Inp = I;
    type Out = O;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        ((self.f)(input), CombiResult::Suc(()))
    }

    fn repr(&self, _: &mut Formatter<'_>) -> Result<(), Error> {
        Ok(())
    }
}

#[allow(non_camel_case_types)]
#[derive_where(Clone; CP: Clone, SP: Clone)]
#[derive_where(Debug; CP: Debug, SP: Debug)]
pub struct pipesuc<CP, SP>(pub CP, pub SP)
where
    CP: Combi,
    SP: Combi<Inp = (CP::Suc, CP::Out), Out = CP::Out, Con = CP::Con, Err = CP::Err>,
    SP::Err: CombiErr<CP::Con>;

impl<CP, SP> Combi for pipesuc<CP, SP>
where
    CP: Combi,
    SP: Combi<Inp = (CP::Suc, CP::Out), Out = CP::Out, Con = CP::Con, Err = CP::Err>,
    SP::Err: CombiErr<CP::Con>,
{
    type Suc = SP::Suc;
    type Err = CP::Err;
    type Con = CP::Con;
    type Inp = CP::Inp;
    type Out = SP::Out;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (cp_out, cp_res) = self.0.comp(input);
        match cp_res {
            CombiResult::Suc(s) => self.1.comp((s, cp_out)),
            CombiResult::Con(c) => (cp_out, CombiResult::Err(CombiErr::catch_con(c))),
            CombiResult::Err(e) => (cp_out, CombiResult::Err(e)),
        }
    }

    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}{}", Repr(&self.0), Repr(&self.1))
    }
}

#[allow(non_camel_case_types)]
#[derive_where(Clone; P1, P2)]
pub struct or<O, P1, P2>(pub P1, pub P2)
where
    P1: Combi<Inp = O, Out = O, Suc = bool>,
    P2: Combi<Inp = O, Out = O, Suc = bool, Con = P1::Con, Err = P1::Err>,
    P1::Err: CombiErr<P1::Con>,
    P1::Con: CombiCon<bool, P1::Con>;

impl<O, P1, P2> Combi for or<O, P1, P2>
where
    P1: Combi<Inp = O, Out = O, Suc = bool>,
    P2: Combi<Inp = O, Out = O, Suc = bool, Con = P1::Con, Err = P1::Err>,
    P1::Err: CombiErr<P1::Con>,
    P1::Con: CombiCon<bool, P1::Con>,
{
    type Suc = bool;
    type Err = P1::Err;
    type Con = P1::Con;
    type Inp = O;
    type Out = O;

    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (p1_out, p1_res) = self.0.comp(input);
        match p1_res {
            CombiResult::Suc(true) => (p1_out, CombiResult::Suc(true)),
            CombiResult::Err(e) => (p1_out, CombiResult::Err(e)),
            CombiResult::Suc(false) => {
                let (p2_out, p2_res) = self.1.comp(p1_out);
                (
                    p2_out,
                    match p2_res {
                        CombiResult::Suc(s) => CombiResult::Suc(s),
                        CombiResult::Err(e) => CombiResult::Err(e),
                        CombiResult::Con(c) => CombiResult::Con(c),
                    },
                )
            }
            CombiResult::Con(c) => {
                let (p2_out, p2_res) = self.1.comp(p1_out);
                (
                    p2_out,
                    match p2_res {
                        CombiResult::Suc(s) => CombiResult::Con(c.combine_suc(s)),
                        CombiResult::Con(c2) => CombiResult::Con(c.combine_con(c2)),
                        CombiResult::Err(e) => CombiResult::Err(e.inherit_con(c)),
                    },
                )
            }
        }
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} or {}", Repr(&self.0), Repr(&self.1))
    }
}
