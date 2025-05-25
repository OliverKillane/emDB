type Bits = usize;
type Bytes = usize;

pub const fn bytes(bits: Bits) -> Bytes {
    (bits + 7) / 8
}

trait DataType {
    const SIZE: Bits;

    fn convert(data: [u8; bytes(Self::SIZE)]) -> Self;
}

pub trait AccessCompTime {
    type Data: DataType;
    const OFFSET: Bits;
    const INDEX: usize;
}

pub trait AccessRunTime {
    type Data: DataType;
    fn offset(&self) -> Bits;
    fn index(&self) -> usize;
}

pub trait Bound {
    const SIZE: Bits;
}

unsafe trait Check<B: Bound> {}

pub trait Cursor {
    type Bound: Bound;
    type Remaining;

    fn advance<B: Bound, const BITS: usize>() -> impl Cursor<Bound = B>;
    fn try_advance<B: Bound>(bits: Bits) -> Option<impl Cursor<Bound = B>>;
}

pub trait ReadCursor: Cursor {
    fn read<P: AccessCompTime>(&self) -> P::Data
    where
        P: Check<Self::Bound>;

    fn try_read<P: AccessRunTime>(&self, pos: &P) -> Option<P::Data>;
}

/* bunch of impls */

impl DataType for u8 {
    const SIZE: Bits = 8;

    fn convert(data: [u8; bytes(Self::SIZE)]) -> Self {
        data[0]
    }
}
