use rayon::prelude::*;
use std::collections::HashMap;

macro_rules! single {
    ($data:ty) => {
        $data
    };
}
macro_rules! stream { ($data:ty) => { impl ParallelIterator<Item = $data> }; }
super::generate_minister_trait! { ParallelOps }

/// ## A very slow (😒) but maximally parallel implementation with [rayon]
/// - Every single operation that can be made a task is sent to the thread pool (massive
///   contention, and overhead for small tasks).
pub struct Parallel;

impl ParallelOps for Parallel {
    type Buffer<Data: Send + Sync> = Vec<Data>;

    fn consume_stream<Data>(iter: impl Iterator<Item = Data>) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        // TODO: Specialise for Range iterator (for which we can efficiently convert directly)
        iter.collect::<Vec<_>>().into_par_iter()
    }

    fn consume_buffer<Data>(buff: Self::Buffer<Data>) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        buff.into_par_iter()
    }

    fn consume_single<Data>(data: Data) -> single!(Data)
    where
        Data: Send + Sync,
    {
        data
    }

    fn export_stream<Data>(stream: stream!(Data)) -> impl Iterator<Item = Data>
    where
        Data: Send + Sync,
    {
        stream.collect::<Vec<_>>().into_iter()
    }

    fn export_buffer<Data>(stream: stream!(Data)) -> Self::Buffer<Data>
    where
        Data: Send + Sync,
    {
        stream.collect::<Vec<_>>()
    }

    fn export_single<Data>(single: single!(Data)) -> Data
    where
        Data: Send + Sync,
    {
        single
    }

    fn error_stream<Data, Error>(
        stream: stream!(Result<Data,Error>),
    ) -> Result<stream!(Data), Error>
    where
        Data: Send + Sync,
        Error: Send + Sync,
    {
        Ok(stream.collect::<Result<Vec<_>, _>>()?.into_par_iter())
    }

    fn error_single<Data, Error>(
        single: single!(Result<Data,Error>),
    ) -> Result<single!(Data), Error>
    where
        Data: Send + Sync,
        Error: Send + Sync,
    {
        single
    }

    type MapStats = ();
    fn map<InData, OutData>(
        stream: stream!(InData),
        mapping: impl Fn(InData) -> OutData + Send + Sync,
        _stats: &Self::MapStats,
    ) -> stream!(OutData)
    where
        InData: Send + Sync,
        OutData: Send + Sync,
    {
        stream.map(mapping)
    }

    type MapSeqStats = ();
    fn map_seq<InData, OutData>(
        stream: stream!(InData),
        mapping: impl FnMut(InData) -> OutData,
        _stats: &Self::MapSeqStats,
    ) -> stream!(OutData)
    where
        InData: Send + Sync,
        OutData: Send + Sync,
    {
        // Cannot work in parallel here - mutating data structures!
        let data = stream.collect::<Vec<_>>();
        data.into_iter()
            .map(mapping)
            .collect::<Vec<_>>()
            .into_par_iter()
    }

    type MapSingleStats = ();
    fn map_single<InData, OutData>(
        single: single!(InData),
        mapping: impl FnOnce(InData) -> OutData,
        _stats: &Self::MapSingleStats,
    ) -> single!(OutData)
    where
        InData: Send + Sync,
        OutData: Send + Sync,
    {
        (mapping)(single)
    }

    type FilterStats = ();
    fn filter<Data>(
        stream: stream!(Data),
        predicate: impl Fn(&Data) -> bool + Send + Sync,
        _stats: &Self::FilterStats,
    ) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        stream.filter(predicate)
    }

    type AllStats = ();
    fn all<Data>(
        stream: stream!(Data),
        predicate: impl Fn(&Data) -> bool + Send + Sync,
        _stats: &Self::AllStats,
    ) -> (bool, stream!(Data))
    where
        Data: Send + Sync,
    {
        let vals = stream.collect::<Vec<_>>();
        let res = vals.par_iter().all(predicate);
        (res, vals.into_par_iter())
    }

    type IsStats = ();
    fn is<Data>(
        single: single!(Data),
        predicate: impl Fn(&Data) -> bool,
        _stats: &Self::IsStats,
    ) -> (bool, single!(Data))
    where
        Data: Send + Sync,
    {
        (predicate(&single), single)
    }

    type CountStats = ();
    fn count<Data>(stream: stream!(Data), _stats: &Self::CountStats) -> single!(usize)
    where
        Data: Send + Sync,
    {
        stream.count()
    }

    type FoldStats = ();
    fn fold<InData, Acc>(
        stream: stream!(InData),
        initial: Acc,
        fold_fn: impl Fn(Acc, InData) -> Acc,
        _stats: &Self::FoldStats,
    ) -> single!(Acc)
    where
        InData: Send + Sync,
        Acc: Send + Sync,
    {
        let mut acc = initial;
        for data in stream.collect::<Vec<_>>() {
            acc = fold_fn(acc, data);
        }
        acc
    }

    type CombineStats = ();
    fn combine<Data>(
        stream: stream!(Data),
        alternative: Data,
        combiner: impl Fn(Data, Data) -> Data + Send + Sync,
        _stats: &Self::CombineStats,
    ) -> single!(Data)
    where
        Data: Send + Sync + Clone,
    {
        stream.reduce(|| alternative.clone(), combiner)
    }

    type SortStats = ();
    fn sort<Data>(
        stream: stream!(Data),
        ordering: impl Fn(&Data, &Data) -> std::cmp::Ordering + Send + Sync,
        _stats: &Self::SortStats,
    ) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        let mut data = stream.collect::<Vec<_>>();
        data.par_sort_unstable_by(ordering);
        data.into_par_iter()
    }

    type TakeStats = ();
    fn take<Data>(stream: stream!(Data), n: usize, _stats: &Self::TakeStats) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        let mut values = stream.collect::<Vec<_>>();
        values.truncate(n);
        values.into_par_iter()
    }

    type GroupByStats = ();
    fn group_by<Key, Rest, Data>(
        stream: stream!(Data),
        split: impl Fn(Data) -> (Key, Rest),
        _stats: &Self::GroupByStats,
    ) -> stream!((Key, stream!(Rest)))
    where
        Data: Send + Sync,
        Key: Eq + std::hash::Hash + Send + Sync,
        Rest: Send + Sync,
    {
        // can improve parallelism
        let mut groups = HashMap::new();
        for data in stream.collect::<Vec<_>>() {
            let (k, r) = split(data);
            groups.entry(k).or_insert_with(Vec::new).push(r);
        }
        groups.into_par_iter().map(|(k, v)| (k, v.into_par_iter()))
    }

    type CrossJoinStats = ();
    fn cross_join<LeftData, RightData>(
        left: stream!(LeftData),
        right: stream!(RightData),
        _stats: &Self::CrossJoinStats,
    ) -> stream!((LeftData, RightData))
    where
        LeftData: Clone + Send + Sync,
        RightData: Clone + Send + Sync,
    {
        let left = left.collect::<Vec<_>>();
        right
            .map(|r| left.par_iter().map(move |l| (l.clone(), r.clone())))
            .flatten()
            .collect::<Vec<_>>()
            .into_par_iter()
    }

    type EquiJoinStats = ();
    fn equi_join<LeftData, RightData, Key>(
        left: stream!(LeftData),
        right: stream!(RightData),
        left_split: impl Fn(&LeftData) -> &Key + Send + Sync,
        right_split: impl Fn(&RightData) -> &Key + Send + Sync,
        _stats: &Self::EquiJoinStats,
    ) -> stream!((LeftData, RightData))
    where
        Key: Eq + std::hash::Hash + Send + Sync,
        LeftData: Clone + Send + Sync,
        RightData: Clone + Send + Sync,
    {
        let left = left.collect::<Vec<_>>();
        let right = right.collect::<Vec<_>>();
        if left.len() < right.len() {
            let mut lefts = HashMap::with_capacity(left.len());
            for l in &left {
                lefts.entry(left_split(l)).or_insert_with(Vec::new).push(l);
            }

            right
                .into_par_iter()
                .filter_map(|r| {
                    lefts.get(right_split(&r)).map(|ls| {
                        ls.par_iter()
                            .map(|l| ((*l).clone(), r.clone()))
                            .collect::<Vec<_>>()
                            .into_par_iter()
                    })
                })
                .flatten()
                .collect::<Vec<_>>()
                .into_par_iter()
        } else {
            let mut rights = HashMap::with_capacity(right.len());
            for r in &right {
                rights
                    .entry(right_split(r))
                    .or_insert_with(Vec::new)
                    .push(r);
            }
            left.into_par_iter()
                .filter_map(|l| {
                    rights.get(left_split(&l)).map(|rs| {
                        rs.par_iter()
                            .map(|r| (l.clone(), (*r).clone()))
                            .collect::<Vec<_>>()
                            .into_par_iter()
                    })
                })
                .flatten()
                .collect::<Vec<_>>()
                .into_par_iter()
        }
    }

    type PredJoinStats = ();
    fn predicate_join<LeftData, RightData>(
        left: stream!(LeftData),
        right: stream!(RightData),
        pred: impl Fn(&LeftData, &RightData) -> bool + Send + Sync,
        _stats: &Self::PredJoinStats,
    ) -> stream!((LeftData, RightData))
    where
        LeftData: Clone + Send + Sync,
        RightData: Clone + Send + Sync,
    {
        let left = left.collect::<Vec<_>>();
        right
            .map(|r| {
                let pred2 = &pred;
                left.par_iter().filter_map(move |l| {
                    if (pred2)(l, &r) {
                        Some((l.clone(), r.clone()))
                    } else {
                        None
                    }
                })
            })
            .flatten()
            .collect::<Vec<_>>()
            .into_par_iter()
    }

    type UnionStats = ();
    fn union<Data>(
        left: stream!(Data),
        right: stream!(Data),
        _stats: &Self::UnionStats,
    ) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        left.chain(right)
    }

    type ForkStats = ();
    fn fork<Data>(stream: stream!(Data), _stats: &Self::ForkStats) -> (stream!(Data), stream!(Data))
    where
        Data: Clone + Send + Sync,
    {
        let (left, right): (Vec<_>, Vec<_>) = stream.map(|d| (d.clone(), d)).unzip();
        (left.into_par_iter(), right.into_par_iter())
    }

    type ForkSingleStats = ();
    fn fork_single<Data>(
        single: single!(Data),
        _stats: &Self::ForkSingleStats,
    ) -> (single!(Data), single!(Data))
    where
        Data: Clone + Send + Sync,
    {
        (single.clone(), single)
    }

    type SplitStats = ();
    fn split<LeftData, RightData>(
        stream: stream!((LeftData, RightData)),
        _stats: &Self::SplitStats,
    ) -> (stream!(LeftData), stream!(RightData))
    where
        LeftData: Send + Sync,
        RightData: Send + Sync,
    {
        let (left, right): (Vec<_>, Vec<_>) = stream.map(|(l, r)| (l, r)).unzip();
        (left.into_par_iter(), right.into_par_iter())
    }
}
