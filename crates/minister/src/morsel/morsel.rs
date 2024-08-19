//! Morsels are operators that can be parallelised (by some runtime determined degree
//! through a [`Splitter`])

use std::{mem, sync::Arc};

use super::{buffer::BufferSpan, datum::Datum, splitter::Splitter, statistics::{Cardinality, Size}};

pub trait Morsel {
    type Data;
    type Statistics;
    const COMPTIME_CARDINALITY: Cardinality;
    fn runtime_cardinality(&self) -> Cardinality;
    fn execute(
        &mut self,
        splitter: &impl Splitter,
    ) -> impl Iterator<Item = impl Iterator<Item = Self::Data>>;
}

pub struct Read<Data> {
    buffer: BufferState<Data>,
}

enum BufferState<Data> {
    Used,
    Available(BufferSpan<Data>),
}

impl<Data> From<Vec<Data>> for Read<Data> {
    fn from(value: Vec<Data>) -> Self {
        Self {
            buffer: BufferState::Available(BufferSpan::from(value)),
        }
    }
}

impl<Data> Morsel for Read<Data> {
    type Data = Data;
    type Statistics = ();

    const COMPTIME_CARDINALITY: Cardinality = Cardinality::Range{
        upper: Size::UnKnown,
        estimate: Size::UnKnown,
        lower: Size::Exact(0),
    };

    fn runtime_cardinality(&self) -> Cardinality {
        todo!()
    }

    fn execute(
        &mut self,
        splitter: &impl Splitter,
    ) -> impl Iterator<Item = impl Iterator<Item = Self::Data>> {
        match mem::replace(&mut self.buffer, BufferState::Used) {
            BufferState::Used => unreachable!("Buffer was used, but execute was called again"),
            BufferState::Available(b) => b.split(splitter),
        }
    }
}

pub struct Map<I, O, M, F>
where
    M: Morsel<Data = I>,
    F: Fn(I) -> O,
{
    morsel: M,
    mapping: F,
}

impl<I, O, M, F> Map<I, O, M, F>
where
    M: Morsel<Data = I>,
    F: Fn(I) -> O,
{
    fn new(morsel: M, mapping: F) -> Self {
        Self { morsel, mapping }
    }
}

impl<I, O, M, F> Morsel for Map<I, O, M, F>
where
    M: Morsel<Data = I>,
    F: Fn(I) -> O,
{
    type Data = O;
    type Statistics = ();

    const COMPTIME_CARDINALITY: Cardinality = M::COMPTIME_CARDINALITY;

    fn runtime_cardinality(&self) -> Cardinality {
        self.morsel.runtime_cardinality()
    }

    fn execute(
        &mut self,
        splitter: &impl Splitter,
    ) -> impl Iterator<Item = impl Iterator<Item = Self::Data>> {
        self.morsel
            .execute(splitter)
            .map(|iter| iter.map(&self.mapping))
    }
}

pub struct Filter<I, M, P> where M: Morsel<Data = I>, P: Fn(&I) -> bool {
    morsel: M,
    predicate: P,
}

impl<I, M, P> Filter<I, M, P> where M: Morsel<Data = I>, P: Fn(&I) -> bool {
    fn new(morsel: M, predicate: P) -> Self {
        Self { morsel, predicate }
    }
}

impl<I, M, P> Morsel for Filter<I, M, P> where M: Morsel<Data = I>, P: Fn(&I) -> bool {
    type Data = I;
    type Statistics = ();

    const COMPTIME_CARDINALITY: Cardinality = M::COMPTIME_CARDINALITY;

    fn runtime_cardinality(&self) -> Cardinality {
        self.morsel.runtime_cardinality()
    }

    fn execute(
        &mut self,
        splitter: &impl Splitter,
    ) -> impl Iterator<Item = impl Iterator<Item = Self::Data>> {
        self.morsel
            .execute(splitter)
            .map(|iter| iter.filter(|data| (self.predicate)(data)))
    }
}

struct Cross;

#[cfg(test)]
mod test {
    use crate::morsel::splitter::SingleSplit;
    use super::Morsel;

    #[test]
    fn foo() {

        let x = vec![1,2,3,4,5,6,7,8,9];

        let read = super::Read::from(x);
        let mut map = super::Map::new(read, |x| x * 2);
        map.execute(&SingleSplit);
    }
}