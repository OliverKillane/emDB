use crate::truth::{Bool, Truth};

// JUSTIFY: type alias instead of a new type
//           - to allow usage in const parameters
pub type Bits = usize;
pub type Bytes = usize;

pub trait Access {
    const SIZE: Bits;
    fn offset(&self) -> Bits;
    fn index(&self) -> usize;
}

pub unsafe trait Check<B: Bound> {}

pub struct BoundConst<const SIZE: Bits>;

unsafe impl<const SIZE: Bits> Bound for BoundConst<SIZE> {
    const SIZE: Bits = SIZE;
}

pub unsafe trait Bound {
    const SIZE: Bits;
}

pub struct AccessConst<const SIZE: Bits, const OFFSET: Bits, const INDEX: usize>;

unsafe impl<B: Bound, const SIZE: Bits, const OFFSET: Bits, const INDEX: usize> Check<B>
    for AccessConst<SIZE, OFFSET, INDEX>
where
    Bool<{ OFFSET + SIZE * (INDEX + 1) < B::SIZE }>: Truth,
{
}

pub struct AccessIndex<const SIZE: Bits, const OFFSET: Bits> {
    pub index: usize,
}
pub struct AccessOffset<const SIZE: Bits, const INDEX: usize> {
    pub offset: Bits,
}
pub struct AccessRuntime<const SIZE: Bits> {
    pub offset: Bits,
    pub index: usize,
}

impl<const SIZE: Bits, const OFFSET: Bits, const INDEX: usize> Access
    for AccessConst<SIZE, OFFSET, INDEX>
{
    const SIZE: Bits = SIZE;

    fn offset(&self) -> Bits {
        OFFSET
    }

    fn index(&self) -> usize {
        INDEX
    }
}

impl<const SIZE: Bits, const OFFSET: Bits> Access for AccessIndex<SIZE, OFFSET> {
    const SIZE: Bits = SIZE;

    fn offset(&self) -> Bits {
        OFFSET
    }

    fn index(&self) -> usize {
        self.index
    }
}

impl<const SIZE: Bits, const INDEX: usize> Access for AccessOffset<SIZE, INDEX> {
    const SIZE: Bits = SIZE;

    fn offset(&self) -> Bits {
        self.offset
    }

    fn index(&self) -> usize {
        INDEX
    }
}

impl<const SIZE: Bits> Access for AccessRuntime<SIZE> {
    const SIZE: Bits = SIZE;

    fn offset(&self) -> Bits {
        self.offset
    }

    fn index(&self) -> usize {
        self.index
    }
}
