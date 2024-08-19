#![allow(clippy::ptr_arg)]

use rayon::{current_num_threads, prelude::*};
use std::collections::HashMap;

macro_rules! single {
    ($data:ty) => {
        $data
    };
}
macro_rules! stream { ($data:ty) => { Vec<Vec<$data>> }; }
super::generate_minister_trait! { ChunkOps }

/// ## A Slow (😒) parallel operator implementation that splits streams into chunks of data.
pub struct Chunk;

fn split_chunks<Data>(
    data_size: usize,
    mut input_data: impl Iterator<Item = Data>,
) -> Vec<Vec<Data>> {
    let num_cores = current_num_threads();
    let chunk_size = data_size / num_cores;
    let mut output_data = Vec::with_capacity(num_cores + 1);

    if let Some(first) = input_data.next() {
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
    }
}

fn merge_chunks<Data>(chunks: Vec<Vec<Data>>) -> Vec<Data> {
    chunks.into_iter().flat_map(|v| v.into_iter()).collect()
}

impl ChunkOps for Chunk {
    fn consume_stream<Data>(iter: impl Iterator<Item = Data>) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        let data = iter.collect::<Vec<_>>();
        split_chunks(data.len(), data.into_iter())
    }

    fn consume_buffer<Data>(buff: Vec<Data>) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        split_chunks(buff.len(), buff.into_iter())
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
        stream.into_iter().flatten()
    }

    fn export_buffer<Data>(stream: stream!(Data)) -> Vec<Data>
    where
        Data: Send + Sync,
    {
        stream.into_iter().flatten().collect()
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
        let mut output = Vec::with_capacity(stream.len());
        for substream in stream {
            output.push(substream.into_iter().collect::<Result<Vec<_>, _>>()?);
        }
        Ok(output)
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

    fn map<InData, OutData>(
        stream: stream!(InData),
        mapping: impl Fn(InData) -> OutData + Send + Sync,
    ) -> stream!(OutData)
    where
        InData: Send + Sync,
        OutData: Send + Sync,
    {
        stream
            .into_par_iter()
            .map(|v| v.into_iter().map(&mapping).collect())
            .collect()
    }

    fn map_seq<InData, OutData>(
        stream: stream!(InData),
        mut mapping: impl FnMut(InData) -> OutData,
    ) -> stream!(OutData)
    where
        InData: Send + Sync,
        OutData: Send + Sync,
    {
        let mut output = Vec::with_capacity(stream.len());
        for substream in stream {
            let mut out_substream = Vec::with_capacity(substream.len());
            for data in substream {
                out_substream.push(mapping(data));
            }
            output.push(out_substream)
        }
        output
    }

    fn map_single<InData, OutData>(
        single: single!(InData),
        mapping: impl FnOnce(InData) -> OutData,
    ) -> single!(OutData)
    where
        InData: Send + Sync,
        OutData: Send + Sync,
    {
        (mapping)(single)
    }

    fn filter<Data>(
        stream: stream!(Data),
        predicate: impl Fn(&Data) -> bool + Send + Sync,
    ) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        stream
            .into_par_iter()
            .map(|v: Vec<Data>| v.into_iter().filter(&predicate).collect())
            .collect()
    }

    fn all<Data>(
        stream: stream!(Data),
        predicate: impl Fn(&Data) -> bool + Send + Sync,
    ) -> (bool, stream!(Data))
    where
        Data: Send + Sync,
    {
        (stream.par_iter().all(|v| v.iter().all(&predicate)), stream)
    }

    fn is<Data>(single: single!(Data), predicate: impl Fn(&Data) -> bool) -> (bool, single!(Data))
    where
        Data: Send + Sync,
    {
        (predicate(&single), single)
    }

    fn count<Data>(stream: stream!(Data)) -> single!(usize)
    where
        Data: Send + Sync,
    {
        stream.into_par_iter().map(|v| v.len()).sum()
    }

    fn fold<InData, Acc>(
        stream: stream!(InData),
        initial: Acc,
        fold_fn: impl Fn(Acc, InData) -> Acc,
    ) -> single!(Acc)
    where
        InData: Send + Sync,
        Acc: Send + Sync,
    {
        let mut acc = initial;
        for substream in stream {
            for data in substream {
                acc = fold_fn(acc, data);
            }
        }
        acc
    }

    fn combine<Data>(
        stream: stream!(Data),
        alternative: Data,
        combiner: impl Fn(Data, Data) -> Data + Send + Sync,
    ) -> single!(Data)
    where
        Data: Send + Sync + Clone,
    {
        stream
            .into_par_iter()
            .filter_map(|v| v.into_iter().reduce(&combiner))
            .reduce(|| alternative.clone(), &combiner)
    }

    fn sort<Data>(
        stream: stream!(Data),
        ordering: impl Fn(&Data, &Data) -> std::cmp::Ordering + Send + Sync,
    ) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        let mut data = stream
            .into_iter()
            .flat_map(|v| v.into_iter())
            .collect::<Vec<_>>();
        data.par_sort_unstable_by(ordering);
        split_chunks(data.len(), data.into_iter())
    }

    fn take<Data>(stream: stream!(Data), n: usize) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        let mut data = merge_chunks(stream);
        data.truncate(n);
        split_chunks(data.len(), data.into_iter())
    }

    fn group_by<Key, Rest, Data>(
        stream: stream!(Data),
        split: impl Fn(Data) -> (Key, Rest),
    ) -> stream!((Key, stream!(Rest)))
    where
        Data: Send + Sync,
        Key: Eq + std::hash::Hash + Send + Sync,
        Rest: Send + Sync,
    {
        let mut groups = HashMap::new();
        for substream in stream {
            for data in substream {
                let (k, r) = split(data);
                groups.entry(k).or_insert_with(Vec::new).push(r);
            }
        }
        split_chunks(
            groups.len(),
            groups
                .into_iter()
                .map(|(k, v)| (k, split_chunks(v.len(), v.into_iter()))),
        )
    }

    fn cross_join<LeftData, RightData>(
        left: stream!(LeftData),
        right: stream!(RightData),
    ) -> stream!((LeftData, RightData))
    where
        LeftData: Clone + Send + Sync,
        RightData: Clone + Send + Sync,
    {
        left.into_par_iter()
            .map(|ls| {
                let mut v = Vec::new();
                for l in ls {
                    for rs in &right {
                        for r in rs {
                            v.push((l.clone(), r.clone()))
                        }
                    }
                }
                v
            })
            .collect::<Vec<_>>()
    }

    /// A very basic optimisation is to hash the smaller side of the join.
    fn equi_join<LeftData, RightData, Key>(
        left: stream!(LeftData),
        right: stream!(RightData),
        left_split: impl Fn(&LeftData) -> &Key + Send + Sync,
        right_split: impl Fn(&RightData) -> &Key + Send + Sync,
    ) -> stream!((LeftData, RightData))
    where
        Key: Eq + std::hash::Hash + Send + Sync,
        LeftData: Clone + Send + Sync,
        RightData: Clone + Send + Sync,
    {
        // NOTE: Not optimised at all, but does mantain balance of chunk sizes
        let left = merge_chunks(left);
        let right = merge_chunks(right);
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
        split_chunks(results.len(), results.into_iter())
    }

    fn predicate_join<LeftData, RightData>(
        left: stream!(LeftData),
        right: stream!(RightData),
        pred: impl Fn(&LeftData, &RightData) -> bool + Send + Sync,
    ) -> stream!((LeftData, RightData))
    where
        LeftData: Clone + Send + Sync,
        RightData: Clone + Send + Sync,
    {
        // NOTE: Can unbalance the chunk sizes
        left.into_par_iter()
            .map(|ls| {
                let mut v = Vec::new();
                for l in ls {
                    for rs in &right {
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
    }

    fn union<Data>(mut left: stream!(Data), right: stream!(Data)) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        left.extend(right);
        left
    }

    fn fork<Data>(stream: stream!(Data)) -> (stream!(Data), stream!(Data))
    where
        Data: Clone + Send + Sync,
    {
        (stream.clone(), stream)
    }

    fn fork_single<Data>(single: single!(Data)) -> (single!(Data), single!(Data))
    where
        Data: Clone + Send + Sync,
    {
        (single.clone(), single)
    }

    fn split<LeftData, RightData>(
        stream: stream!((LeftData, RightData)),
    ) -> (stream!(LeftData), stream!(RightData))
    where
        LeftData: Send + Sync,
        RightData: Send + Sync,
    {
        stream
            .into_iter()
            .map(|inner| inner.into_iter().unzip())
            .unzip()
    }
}
