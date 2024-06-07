use std::collections::HashMap;

use super::Physical;

/// Extremely basic push operators.
pub struct Basic;

impl Physical for Basic {
    type Stream<Data> = Vec<Data>;
    type Single<Data> = Data;

    fn consume_stream<Data>(iter: impl Iterator<Item = Data>) -> Self::Stream<Data> {
        iter.collect()
    }
    fn consume_single<Data>(data: Data) -> Self::Single<Data> {
        data
    }

    fn export_stream<Data>(stream: Self::Stream<Data>) -> impl Iterator<Item = Data> {
        stream.into_iter()
    }
    fn export_single<Data>(single: Self::Single<Data>) -> Data {
        single
    }

    fn map<InData, OutData>(
        stream: Self::Stream<InData>,
        mapping: impl Fn(InData) -> OutData,
    ) -> Self::Stream<OutData> {
        stream.into_iter().map(mapping).collect()
    }

    fn filter<Data>(
        stream: Self::Stream<Data>,
        predicate: impl Fn(&Data) -> bool,
    ) -> Self::Stream<Data> {
        stream.into_iter().filter(|data| predicate(data)).collect()
    }

    fn fold<InData, Acc>(
        stream: Self::Stream<InData>,
        initial: Acc,
        fold_fn: impl Fn(Acc, InData) -> Acc,
    ) -> Self::Single<Acc> {
        let mut acc = initial;
        for data in stream {
            acc = fold_fn(acc, data);
        }
        acc
    }

    fn combine<Data>(
        stream: Self::Stream<Data>,
        combiner: impl Fn(Data, Data) -> Data,
    ) -> Self::Single<Data> {
        stream.into_iter().reduce(combiner).unwrap()
    }

    fn sort<Data>(
        mut stream: Self::Stream<Data>,
        ordering: impl FnMut(&Data, &Data) -> std::cmp::Ordering,
    ) -> Self::Stream<Data> {
        stream.sort_by(ordering);
        stream
    }

    fn take<Data>(mut stream: Self::Stream<Data>, n: usize) -> Self::Stream<Data> {
        stream.truncate(n);
        stream
    }

    fn group_by<Key, Rest>(
        stream: Self::Stream<(Key, Rest)>,
    ) -> Self::Stream<(Key, Self::Stream<Rest>)>
    where
        Key: Eq + std::hash::Hash,
    {
        let mut groups = HashMap::new();
        for (k, r) in stream {
            groups.entry(k).or_insert_with(Vec::new).push(r);
        }
        groups.into_iter().collect()
    }

    fn cross_join<LeftData, RightData>(
        left: Self::Stream<LeftData>,
        right: Self::Stream<RightData>,
    ) -> Self::Stream<(LeftData, RightData)>
    where
        LeftData: Clone,
        RightData: Clone,
    {
        let mut result = Vec::with_capacity(left.len() * right.len());
        for l in left {
            for r in &right {
                result.push((l.clone(), r.clone()));
            }
        }
        result
    }

    fn equi_join<LeftData, RightData, Key>(
        left: Self::Stream<LeftData>,
        right: Self::Stream<RightData>,
        left_split: impl Fn(&LeftData) -> &Key,
        right_split: impl Fn(&RightData) -> &Key,
    ) -> Self::Stream<(LeftData, RightData)>
    where
        Key: Eq + std::hash::Hash,
        LeftData: Clone,
        RightData: Clone,
    {
        // TODO: check for smallest size
        let mut lefts = HashMap::new();
        for l in &left {
            lefts.entry(left_split(l)).or_insert_with(Vec::new).push(l);
        }
        let mut results = Vec::with_capacity(left.len() * right.len());
        for r in right {
            if let Some(ls) = lefts.get(right_split(&r)) {
                for l in ls {
                    results.push(((*l).clone(), r.clone()))
                }
            }
        }
        results
    }

    fn predicate_join<LeftData, RightData>(
        left: Self::Stream<LeftData>,
        right: Self::Stream<RightData>,
        pred: impl Fn(&LeftData, &RightData) -> bool,
    ) -> Self::Stream<(LeftData, RightData)>
    where
        LeftData: Clone,
        RightData: Clone,
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

    fn union<Data>(mut left: Self::Stream<Data>, right: Self::Stream<Data>) -> Self::Stream<Data> {
        left.extend(right);
        left
    }

    fn fork<Data>(stream: &Self::Stream<Data>) -> Self::Stream<Data>
    where
        Data: Clone,
    {
        stream.clone()
    }

    fn split<LeftData, RightData>(
        stream: Self::Stream<(LeftData, RightData)>,
    ) -> (Self::Stream<LeftData>, Self::Stream<RightData>) {
        stream.into_iter().unzip()
    }

    fn error_stream<Data, Error>(
        stream: Self::Stream<Result<Data, Error>>,
    ) -> Result<Self::Stream<Data>, Error> {
        stream.into_iter().collect::<Result<_, _>>()
    }

    fn error_single<Data, Error>(
        single: Self::Single<Result<Data, Error>>,
    ) -> Result<Self::Single<Data>, Error> {
        single
    }

    fn map_seq<InData, OutData>(
        stream: Self::Stream<InData>,
        mapping: impl FnMut(InData) -> OutData,
    ) -> Self::Stream<OutData> {
        stream.into_iter().map(mapping).collect()
    }

    fn map_single<InData, OutData>(
        single: Self::Single<InData>,
        mapping: impl FnOnce(InData) -> OutData,
    ) -> Self::Single<OutData> {
        (mapping)(single)
    }

    fn count<Data>(stream: Self::Stream<Data>) -> Self::Single<usize> {
        stream.len()
    }
    
    fn fork_single<Data>(single: &Self::Single<Data>) -> Self::Single<Data>
        where
            Data: Clone {
        single.clone()
    }
}
