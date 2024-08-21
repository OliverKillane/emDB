//! Morsels are operators that can be parallelised (by some runtime determined degree
//! through a [`Splitter`])

use super::{
    buffer::BufferSpan,
    datum::Datum,
    splitter::Splitter,
    statistics::{combine, reduce, Cardinality, Estimate},
};

/// As per [`Morsel::work`].
const CONSUME_MSG: &'static str = "Operator has been consumed";

pub trait Morsel: Send + Sync {
    type Data: Send;
    type Statistics: Sync;
    const COMPTIME_CARDINALITY: Cardinality;
    fn runtime_cardinality(&self) -> Cardinality;
    fn estimate_cardinality(&self) -> Estimate;

    /// Produces the [`Iterator`]s for the work to execute the operator
    /// INV: Can only be called once per object.
    ///
    /// NOTE: Why not just consume `self` and prevent re-calling.
    ///       Because we want to share some state across threads, and cannot do
    ///       so without capturing the state in a closure, or using a more complex
    ///       & harder to pptimise heap allocation (e.g. of an `Arc<dyn Fn()` ).
    ///
    ///       Those captures have the lifetime of the operator, so we need it
    ///       around until execution of the work is complete
    fn work(
        &mut self,
        splitter: &impl Splitter,
    ) -> impl Iterator<Item = impl Iterator<Item = Self::Data> + Send + Sync>;
}

pub struct Read<Data> {
    buffer: Option<BufferSpan<Data>>,
}

impl<Data> From<Vec<Data>> for Read<Data> {
    fn from(value: Vec<Data>) -> Self {
        Self {
            buffer: Some(BufferSpan::from(value)),
        }
    }
}

impl<Data> Read<Data> {
    pub fn len(&self) -> usize {
        self.buffer.as_ref().expect(CONSUME_MSG).len()
    }
}

impl<Data> Morsel for Read<Data>
where
    Data: Send + Sync,
{
    type Data = Data;
    type Statistics = ();
    const COMPTIME_CARDINALITY: Cardinality = Cardinality::Unknown;

    fn runtime_cardinality(&self) -> Cardinality {
        Cardinality::Exact(self.len())
    }

    fn estimate_cardinality(&self) -> Estimate {
        Estimate {
            size: self.len(),
            work: 1,
            confidence: 0,
        }
    }

    fn work(
        &mut self,
        splitter: &impl Splitter,
    ) -> impl Iterator<Item = impl Iterator<Item = Self::Data> + Send + Sync> {
        self.buffer.take().expect(CONSUME_MSG).split(splitter)
    }
}

pub struct ReadBorrow<Data> {
    buffer: Option<Vec<Datum<Data>>>,
}

impl<Data> From<Vec<Data>> for ReadBorrow<Data> {
    fn from(value: Vec<Data>) -> Self {
        Self {
            buffer: Some(value.into_iter().map(Datum::new).collect()),
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
    F: Fn(I) -> O + Send + Sync,
    O: Send,
{
    type Data = O;
    type Statistics = ();

    const COMPTIME_CARDINALITY: Cardinality = M::COMPTIME_CARDINALITY;

    fn runtime_cardinality(&self) -> Cardinality {
        self.morsel.runtime_cardinality()
    }

    fn estimate_cardinality(&self) -> Estimate {
        // TODO: some measure of work needs to be added here (e.g. from statistics)
        self.morsel.estimate_cardinality()
    }

    fn work(
        &mut self,
        splitter: &impl Splitter,
    ) -> impl Iterator<Item = impl Iterator<Item = Self::Data> + Send + Sync> {
        self.morsel
            .work(splitter)
            .map(|iter| iter.map(|c| (self.mapping)(c)))
    }
}

pub struct Filter<I, M, P>
where
    M: Morsel<Data = I>,
    P: Fn(&I) -> bool,
{
    morsel: M,
    predicate: P,
}

impl<I, M, P> Filter<I, M, P>
where
    M: Morsel<Data = I>,
    P: Fn(&I) -> bool,
{
    fn new(morsel: M, predicate: P) -> Self {
        Self { morsel, predicate }
    }
}

impl<I, M, P> Morsel for Filter<I, M, P>
where
    M: Morsel<Data = I>,
    P: Fn(&I) -> bool + Send + Sync,
    I: Send,
{
    type Data = I;
    type Statistics = ();
    const COMPTIME_CARDINALITY: Cardinality = reduce(M::COMPTIME_CARDINALITY);

    fn runtime_cardinality(&self) -> Cardinality {
        reduce(self.morsel.runtime_cardinality())
    }

    fn estimate_cardinality(&self) -> Estimate {
        self.morsel.estimate_cardinality()
    }

    fn work(
        &mut self,
        splitter: &impl Splitter,
    ) -> impl Iterator<Item = impl Iterator<Item = Self::Data> + Send + Sync> {
        // NOTE: Because filter takes an FnMut (why?) we need to create unique
        //       `FnMut`s, that can all borrow the same `Fn`
        self.morsel
            .work(splitter)
            .map(|iter| iter.filter(|x| (self.predicate)(x)))
    }
}

struct Union<Data, LM, RM>
where
    LM: Morsel<Data = Data>,
    RM: Morsel<Data = Data>,
{
    left: LM,
    right: RM,
}

impl <Data, LM, RM> Union<Data, LM, RM> where
LM: Morsel<Data = Data>,
RM: Morsel<Data = Data>, {
    fn new(left: LM, right: RM) -> Self {
        Self { left, right }
    }
}

impl <Data, LM, RM> Morsel for Union<Data, LM, RM> where
Data: Send,
LM: Morsel<Data = Data>,
RM: Morsel<Data = Data>  {
    type Data = Data;
    type Statistics = ();

    const COMPTIME_CARDINALITY: Cardinality = combine(LM::COMPTIME_CARDINALITY, RM::COMPTIME_CARDINALITY);

    fn runtime_cardinality(&self) -> Cardinality {
        combine(self.left.runtime_cardinality(), self.right.runtime_cardinality())
    }

    fn estimate_cardinality(&self) -> Estimate {
        todo!()
    }

    /// To keep work even, zips together the iterators from each
    /// INV: the splitting returns the same number of splits.
    fn work(
        &mut self,
        splitter: &impl Splitter,
    ) -> impl Iterator<Item = impl Iterator<Item = Self::Data> + Send + Sync> {
        self.left.work(splitter).zip(self.right.work(splitter)).map(|(l, r)| l.chain(r))
    }
}

#[cfg(test)]
mod test {
    use super::Morsel;
    use crate::morsel::splitter::{self, SingleSplit};

    #[test]
    fn foo() {
        let x = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let y = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let read_x = super::Read::from(x);
        let map = super::Map::new(read_x, |x| x * 2);
        let filt = super::Filter::new(map, |x| x % 2 == 0);

        let read_y = super::Read::from(y);
        let map_y = super::Map::new(read_y, |x| x + 100);

        let mut union = super::Union::new(filt, map_y);

        union.work(&splitter::SingleSplit).for_each(|iter| {
            iter.for_each(|x| {
                println!("{}", x);
            });
        });
    }
}
