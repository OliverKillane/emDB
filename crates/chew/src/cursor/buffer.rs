use std::{marker::PhantomData};

use super::{Bound, CheckAccess, Cursor, Reader};
use crate::{
    access,
    data::Data,
    utils::size::{Bits, Bytes, bytes},
};

// TODO: Buffer with runtime offset
pub struct Buffer<'brw, B: Bound, const OFFSET: Bits> {
    buffer: &'brw [u8],
    _phantom: PhantomData<B>,
}

impl<'brw, B: Bound, const OFFSET: Bits> Cursor for Buffer<'brw, B, OFFSET> {
    type Bound = B;
}

impl<'brw, B: Bound, const OFFSET: Bits> Reader for Buffer<'brw, B, OFFSET> {
    fn read<P: access::CompTime>(&self) -> P::Data
    where
        P: CheckAccess<Self::Bound>,
        [(); bytes(P::Data::SIZE)]:,
    {
        let bit_start = OFFSET + P::OFFSET + (P::Data::SIZE * P::INDEX);
        let shift = bit_start % 8;
        let byte_start = bit_start / 8;
        let byte_end = (OFFSET + P::OFFSET + (P::Data::SIZE * (P::INDEX + 1))) / 8;

        unimplemented!()
    }

    fn try_read<P: access::RunTime>(&self, pos: &P) -> Option<P::Data> {
        unimplemented!()
    }
}

/*

fn mod type {
    reader()

    impl reader {
        fn x(&self) -> reader2 {
             
        }
    }
    writer
    value
}

msg {
    some value etc
}


Each instance has a &[] covering just it.

Each stage goes to a type
 - instance has the actual reference to data
 - each contains the data they need (e.g. length, number of repetitions, which variant)

instance: (needs the cursor to the buffer)
repeat: a vec of the stages{
    justify: small size, expect the stages to be large.
    for the case with identically sizes stages, just use an array?!
}
choice: (
    an enum, with the chosen stage
)
until: (
    same as repeat
)
series: (
    a struct of stages
)

So we can build up a struct containing the stages as members.
Each instance is just:
   &[]
   a bunch of type accesses, using the &[]
   We can also optimise the &[] to be &[:CONST]

Each Type access has its own cursor access.
 - why cursor and not span: because we want control (e.g. cursor span mmapped io)
*/