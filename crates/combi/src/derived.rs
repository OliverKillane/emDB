//! [Combi]s derived by function from the [core] combinators.

use crate::{
    core::{lift, manyappsep, mapsuc, pipesuc},
    Combi, CombiCon, CombiErr,
};

/// Use the `s` selection [Combi] to determine if `p` should be applied, continues until `s` succeeds with false.
/// ```text
/// S P S P S P ... S <- false
/// ```
#[inline]
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

/// Apply the item parser, then apply [many0], collecting all the results.
/// ```text
/// P S P S P S P ... S <- false
/// ```
#[inline]
pub fn many1<O, SP, IP>(
    sep: SP,
    item: IP,
) -> impl Combi<Inp = O, Out = O, Suc = Vec<IP::Suc>, Con = SP::Con, Err = SP::Err>
where
    SP: Combi<Inp = O, Out = O, Suc = bool>,
    IP: Combi<Inp = O, Out = O, Con = SP::Con, Err = SP::Err> + Clone,
    SP::Con: CombiCon<IP::Suc, SP::Con>,
    SP::Err: CombiErr<SP::Con>,
{
    pipesuc(mapsuc(item.clone(), |a| vec![a]), manyappsep(sep, item))
}
