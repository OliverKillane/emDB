use std::marker::PhantomData;

use crate::{data::Data, utils::size::Bits};

pub trait CompTime {
    type Data: Data;
    const OFFSET: Bits;
    const INDEX: usize;
}

pub trait RunTime {
    type Data: Data;
    fn offset(&self) -> Bits;
    fn index(&self) -> usize;
}

impl<C: CompTime> RunTime for C {
    type Data = C::Data;

    fn offset(&self) -> Bits {
        C::OFFSET
    }

    fn index(&self) -> usize {
        C::INDEX
    }
}

pub struct CompTimeAll<D: Data, const OFFSET: Bits, const INDEX: usize>(PhantomData<D>);

impl<D: Data, const OFFSET: Bits, const INDEX: usize> CompTime for CompTimeAll<D, OFFSET, INDEX>
where
    D: Data,
{
    type Data = D;
    const OFFSET: Bits = OFFSET;
    const INDEX: usize = INDEX;
}

pub struct CompTimeOffset<D: Data, const OFFSET: Bits> {
    _phantom: PhantomData<D>,
    pub index: usize,
}

impl<D: Data, const OFFSET: Bits> CompTimeOffset<D, OFFSET>
where
    D: Data,
{
    pub fn new(index: usize) -> Self {
        Self {
            _phantom: PhantomData,
            index,
        }
    }
}

impl<D: Data, const OFFSET: Bits> RunTime for CompTimeOffset<D, OFFSET>
where
    D: Data,
{
    type Data = D;

    fn offset(&self) -> Bits {
        OFFSET
    }

    fn index(&self) -> usize {
        self.index
    }
}

pub struct CompTimeIndex<D: Data, const INDEX: usize> {
    _phantom: PhantomData<D>,
    pub offset: Bits,
}

impl<D: Data, const INDEX: usize> CompTimeIndex<D, INDEX>
where
    D: Data,
{
    pub fn new(offset: Bits) -> Self {
        Self {
            _phantom: PhantomData,
            offset,
        }
    }
}

impl<D: Data, const INDEX: usize> RunTime for CompTimeIndex<D, INDEX>
where
    D: Data,
{
    type Data = D;

    fn offset(&self) -> Bits {
        self.offset
    }

    fn index(&self) -> usize {
        INDEX
    }
}
