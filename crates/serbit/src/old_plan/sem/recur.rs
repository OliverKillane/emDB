// TODO: use indexset, traverse graph and find any recursive nodes

use super::*;
use crate::plan::ir;

pub struct Recur;

pub enum PathNode<'items, 'ints, 'bools> {
    Item(ir::keys::Item<'items>),
    Bool(ir::keys::Bool<'bools>),
    Int(ir::keys::Int<'ints>),
}

pub enum Error<'items, 'ints, 'bools> {
    Infinite { path: Vec<PathNode<'items, 'ints, 'bools>> },
}

impl<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N: ir::Namer>
    Semantic<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N> for Recur
{
    type Error = Error<'items, 'ints, 'bools>;

    fn analyse(
        plan: &ir::Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N>,
    ) -> Result<(), Self::Error> {
        unimplemented!()        
    }
}

