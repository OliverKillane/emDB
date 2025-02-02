#![allow(clippy::ptr_arg)]
use std::collections::HashMap;

macro_rules! single {
    ($data:ty) => {
        $data
    };
}
macro_rules! stream { ($data:ty) => { Vec<$data> }; }
super::generate_minister_trait! { BasicOps }

/// ## An extremely basic push operator implementation.
/// - Designed to be as correct as possible
/// - Simple implementation pushed values between vectors
/// - No extra wrapping - it is literally just vectors
///
/// This implementation is easy to understand, and very clearly correct.
pub struct Basic;

impl BasicOps for Basic {
    type Buffer<Data: Send + Sync> = Vec<Data>;

    fn consume_stream<Data>(iter: impl Iterator<Item = Data>) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        iter.collect()
    }

    fn consume_buffer<Data>(buff: Vec<Data>) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        buff
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
        stream.into_iter()
    }

    fn export_buffer<Data>(stream: stream!(Data)) -> Vec<Data>
    where
        Data: Send + Sync,
    {
        stream
    }
    fn export_single<Data>(single: single!(Data)) -> Data
    where
        Data: Send + Sync,
    {
        single
    }

    fn error_stream<Data, Error>(
        stream: stream!(Result<Data, Error>),
    ) -> Result<stream!(Data), Error>
    where
        Data: Send + Sync,
        Error: Send + Sync,
    {
        stream.into_iter().collect::<Result<_, _>>()
    }

    fn error_single<Data, Error>(
        single: single!(Result<Data, Error>),
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
        stream.into_iter().map(mapping).collect()
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
        stream.into_iter().map(mapping).collect()
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
        stream.into_iter().filter(|data| predicate(data)).collect()
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
        for data in &stream {
            if !predicate(data) {
                return (false, stream);
            }
        }
        (true, stream)
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
        stream.len()
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
        for data in stream {
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
        stream.into_iter().reduce(combiner).unwrap_or(alternative)
    }

    type SortStats = ();
    fn sort<Data>(
        mut stream: stream!(Data),
        ordering: impl Fn(&Data, &Data) -> std::cmp::Ordering + Send + Sync,
        _stats: &Self::SortStats,
    ) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        stream.sort_unstable_by(ordering);
        stream
    }

    type TakeStats = ();
    fn take<Data>(mut stream: stream!(Data), n: usize, _stats: &Self::TakeStats) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        stream.truncate(n);
        stream
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
        let mut groups = HashMap::new();
        for data in stream {
            let (k, r) = split(data);
            groups.entry(k).or_insert_with(Vec::new).push(r);
        }
        groups.into_iter().collect()
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
        let mut result = Vec::with_capacity(left.len() * right.len());
        for l in left {
            for r in &right {
                result.push((l.clone(), r.clone()));
            }
        }
        result
    }

    /// A very basic optimisation is to hash the smaller side of the join.
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
        let mut results = Vec::with_capacity(left.len() * right.len());
        if left.len() < right.len() {
            let mut lefts = HashMap::with_capacity(left.len());
            for l in &left {
                lefts.entry(left_split(l)).or_insert_with(Vec::new).push(l);
            }
            for r in right {
                if let Some(ls) = lefts.get(right_split(&r)) {
                    for l in ls {
                        results.push(((*l).clone(), r.clone()))
                    }
                }
            }
        } else {
            let mut rights = HashMap::with_capacity(right.len());
            for r in &right {
                rights
                    .entry(right_split(r))
                    .or_insert_with(Vec::new)
                    .push(r);
            }
            for l in left {
                if let Some(rs) = rights.get(left_split(&l)) {
                    for r in rs {
                        results.push((l.clone(), (*r).clone()))
                    }
                }
            }
        }
        results
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
        let mut results = Vec::with_capacity(left.len() * right.len());
        for l in &left {
            for r in &right {
                if pred(l, r) {
                    results.push((l.clone(), r.clone()));
                }
            }
        }
        results
    }

    type UnionStats = ();
    fn union<Data>(
        mut left: stream!(Data),
        right: stream!(Data),
        _stats: &Self::UnionStats,
    ) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        left.extend(right);
        left
    }

    type ForkStats = ();
    fn fork<Data>(stream: stream!(Data), _stats: &Self::ForkStats) -> (stream!(Data), stream!(Data))
    where
        Data: Clone + Send + Sync,
    {
        (stream.clone(), stream)
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
        stream.into_iter().unzip()
    }
}
