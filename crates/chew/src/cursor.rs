use crate::access::{Access, Bits, Bound, BoundConst, Bytes, Check};

// JUSTIFY: type alias instead of a new type
//           - prevent having to pass bound for SIZE > 0
type RawBytes<P: Access> = [u8; bytes(P::SIZE)];

pub const fn bytes(bits: Bits) -> Bytes {
    (bits + 7) / 8
}

pub trait ReadCursor {
    type Bound: Bound;
    fn read<P: Access>(&self, pos: &P) -> RawBytes<P>
    where
        P: Check<Self::Bound>;

    fn try_read<P: Access>(&self, pos: &P) -> Option<RawBytes<P>>;
}

pub struct ReadBufferConst<'brw, const OFFSET: Bits, const SIZE: Bits> {
    buffer: &'brw [u8],
}

impl <'brw, const OFFSET: Bits, const SIZE: Bits> ReadCursor for ReadBufferConst<'brw, OFFSET, SIZE> {
    type Bound = BoundConst<SIZE>;

    fn read<P: Access>(&self, pos: &P) -> RawBytes<P>
    where
        P: Check<Self::Bound> {
        let offset = pos.offset() + OFFSET;
        let byte = offset / 8;
        let bit = offset % 8;
        let mut result = [0u8; bytes(P::SIZE)]
        result
    }

    fn try_read<P: Access>(&self, pos: &P) -> Option<RawBytes<P>> {
        todo!()
    }
}

