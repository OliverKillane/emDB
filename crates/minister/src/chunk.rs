#![allow(clippy::ptr_arg)]
use rayon::{current_num_threads, prelude::*};
use std::{collections::HashMap, iter::FlatMap, vec::IntoIter};

macro_rules! single {
    ($data:ty) => {
        $data
    };
}
macro_rules! stream { ($data:ty) => { ChunkVecs<$data> }; }
super::generate_minister_trait! { ChunkOps }

/// ## A Slow (😒) parallel operator implementation that splits streams into chunks of data.
pub struct Chunk;

#[derive(Clone)]
pub struct ChunkVecs<Data> {
    chunks: Vec<Vec<Data>>,
}

impl<Data> ChunkVecs<Data> {
    fn merge_chunks(self) -> Vec<Data> {
        self.chunks
            .into_iter()
            .flat_map(|v| v.into_iter())
            .collect()
    }

    fn split_chunks(data_size: usize, mut input_data: impl Iterator<Item = Data>) -> Self {
        let num_cores = current_num_threads();
        let chunk_size = data_size / num_cores;
        let mut output_data = Vec::with_capacity(num_cores + 1);

        Self {
            chunks: if let Some(first) = input_data.next() {
                let mut first_vec = Vec::with_capacity(chunk_size);
                first_vec.push(first);
                output_data.push(first_vec);

                let mut current_chunk_size = 1;
                for data in input_data {
                    if current_chunk_size == chunk_size {
                        let mut next_vec = Vec::with_capacity(chunk_size);
                        next_vec.push(data);
                        output_data.push(next_vec);
                    } else {
                        output_data.last_mut().unwrap().push(data);
                    }
                    current_chunk_size += 1;
                }
                output_data
            } else {
                debug_assert_eq!(data_size, 0);
                output_data
            },
        }
    }
}

impl<Data> From<Vec<Data>> for ChunkVecs<Data> {
    fn from(value: Vec<Data>) -> Self {
        ChunkVecs::from(vec![value])
    }
}

impl<Data> From<Vec<Vec<Data>>> for ChunkVecs<Data> {
    fn from(value: Vec<Vec<Data>>) -> Self {
        ChunkVecs { chunks: value }
    }
}

impl<Data> IntoIterator for ChunkVecs<Data> {
    type Item = Data;
    type IntoIter = FlatMap<
        IntoIter<Vec<Data>>,
        IntoIter<Data>,
        fn(Vec<Data>) -> <Vec<Data> as IntoIterator>::IntoIter,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.chunks.into_iter().flat_map(Vec::into_iter)
    }
}

impl ChunkOps for Chunk {
    type Buffer<Data: Send + Sync> = ChunkVecs<Data>;

    fn consume_stream<Data>(iter: impl Iterator<Item = Data>) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        let data = iter.collect::<Vec<_>>();
        ChunkVecs::split_chunks(data.len(), data.into_iter())
    }

    fn consume_buffer<Data>(buff: Self::Buffer<Data>) -> stream!(Data)
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

    fn export_buffer<Data>(stream: stream!(Data)) -> Self::Buffer<Data>
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
        stream
            .chunks
            .into_par_iter()
            .map(|vs| vs.into_iter().collect::<Result<Vec<_>, _>>())
            .collect::<Result<Vec<Vec<_>>, _>>()
            .map(ChunkVecs::from)
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
        stream
            .chunks
            .into_par_iter()
            .map(|v| v.into_iter().map(&mapping).collect::<Vec<_>>())
            .collect::<Vec<_>>()
            .into()
    }

    type MapSeqStats = ();
    fn map_seq<InData, OutData>(
        stream: stream!(InData),
        mut mapping: impl FnMut(InData) -> OutData,
        _stats: &Self::MapSeqStats,
    ) -> stream!(OutData)
    where
        InData: Send + Sync,
        OutData: Send + Sync,
    {
        let mut output = Vec::with_capacity(stream.chunks.len());
        for substream in stream.chunks {
            let mut out_substream = Vec::with_capacity(substream.len());
            for data in substream {
                out_substream.push(mapping(data));
            }
            output.push(out_substream)
        }
        output.into()
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
        stream
            .chunks
            .into_par_iter()
            .map(|v: Vec<Data>| v.into_iter().filter(&predicate).collect::<Vec<_>>())
            .collect::<Vec<_>>()
            .into()
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
        (
            stream.chunks.par_iter().all(|v| v.iter().all(&predicate)),
            stream,
        )
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
    fn count<Data>(
        stream: stream!(Data),
        _stats: &Self::CountStats,
    ) -> single!(usize)
    where
        Data: Send + Sync,
    {
        stream.chunks.into_par_iter().map(|v| v.len()).sum()
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
        for substream in stream.chunks {
            for data in substream {
                acc = fold_fn(acc, data);
            }
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
        stream
            .chunks
            .into_par_iter()
            .filter_map(|v| v.into_iter().reduce(&combiner))
            .reduce(|| alternative.clone(), &combiner)
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
        let mut data = stream
            .chunks
            .into_iter()
            .flat_map(|v| v.into_iter())
            .collect::<Vec<_>>();
        data.par_sort_unstable_by(ordering);
        ChunkVecs::split_chunks(data.len(), data.into_iter())
    }

    type TakeStats = ();
    fn take<Data>(
        stream: stream!(Data), 
        n: usize,
        _stats: &Self::TakeStats,
    ) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        let mut data = stream.merge_chunks();
        data.truncate(n);
        ChunkVecs::split_chunks(data.len(), data.into_iter())
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
        for substream in stream.chunks {
            for data in substream {
                let (k, r) = split(data);
                groups.entry(k).or_insert_with(Vec::new).push(r);
            }
        }
        ChunkVecs::split_chunks(
            groups.len(),
            groups
                .into_iter()
                .map(|(k, v)| (k, ChunkVecs::split_chunks(v.len(), v.into_iter()))),
        )
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
        left.chunks
            .into_par_iter()
            .map(|ls| {
                let mut v = Vec::new();
                for l in ls {
                    for rs in &right.chunks {
                        for r in rs {
                            v.push((l.clone(), r.clone()))
                        }
                    }
                }
                v
            })
            .collect::<Vec<_>>()
            .into()
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
        // NOTE: Not optimised at all, but does mantain balance of chunk sizes
        let left = left.merge_chunks();
        let right = right.merge_chunks();
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
        ChunkVecs::split_chunks(results.len(), results.into_iter())
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
        // NOTE: Can unbalance the chunk sizes
        left.chunks
            .into_par_iter()
            .map(|ls| {
                let mut v = Vec::new();
                for l in ls {
                    for rs in &right.chunks {
                        for r in rs {
                            if pred(&l, r) {
                                v.push((l.clone(), r.clone()))
                            }
                        }
                    }
                }
                v
            })
            .collect::<Vec<_>>()
            .into()
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
        left.chunks.extend(right.chunks);
        left
    }

    type ForkStats = ();
    fn fork<Data>(
        stream: stream!(Data),
        _stats: &Self::ForkStats,
    ) -> (stream!(Data), stream!(Data))
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
        let (left, right) = stream
            .chunks
            .into_par_iter()
            .map(|inner| inner.into_iter().collect::<(Vec<_>, Vec<_>)>())
            .collect::<Vec<_>>()
            .into_iter().collect::<(Vec<_>, Vec<_>)>();
        (left.into(), right.into())
    }
}
