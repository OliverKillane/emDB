//! Use agnostic parsers that can be derived from the [crate::core] [Combis](Combi).

use crate::{
    core::{lift, manyappsep},
    Combi, CombiCon, CombiErr,
};

pub fn many0<O, SP, IP>(
    s: SP,
    p: IP,
) -> impl Combi<Inp = O, Out = O, Suc = Vec<IP::Suc>, Err = SP::Err, Con = SP::Con>
where
    SP: Combi<Inp = O, Out = O, Suc = bool>,
    IP: Combi<Inp = O, Out = O, Con = SP::Con, Err = SP::Err>,
    SP::Con: CombiCon<IP::Suc, SP::Con>,
    SP::Err: CombiErr<SP::Con>,
{
    lift(manyappsep(s, p), |i| (Vec::new(), i), |o| o)
}
