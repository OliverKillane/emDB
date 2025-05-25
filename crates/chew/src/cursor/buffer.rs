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
