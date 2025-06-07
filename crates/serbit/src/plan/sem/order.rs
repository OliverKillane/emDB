// TODO: use indexset, traverse graph and find any recursive nodes

use super::*;
use crate::plan::ir;

pub struct Order;

// Where can we re-use a value 
// Context
/*
int i;
x[u8; i]
*/

pub enum Error {
    UsedEarly {
        
    }
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

