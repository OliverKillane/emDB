//! Combinators dealing with boolean [success](Combi::Suc) types.
use crate::{Combi, CombiCon, CombiErr, CombiResult, Repr};
use derive_where::derive_where;

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

    #[inline]
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

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct not<P>(pub P)
where
    P: Combi<Suc = bool>;

impl<P> Combi for not<P>
where
    P: Combi<Suc = bool>,
{
    type Suc = bool;
    type Err = P::Err;
    type Con = P::Con;
    type Inp = P::Inp;
    type Out = P::Out;

    #[inline]
    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>) {
        let (p_out, p_res) = self.0.comp(input);
        (
            p_out,
            match p_res {
                CombiResult::Suc(s) => CombiResult::Suc(!s),
                CombiResult::Con(c) => CombiResult::Con(c),
                CombiResult::Err(e) => CombiResult::Err(e),
            },
        )
    }

    fn repr(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "~{}", Repr(&self.0))
    }
}
