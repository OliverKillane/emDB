use std::collections::HashMap;

macro_rules! single {
    ($data:ty) => {
        $data
    };
}
macro_rules! stream { ($data:ty) => { impl Iterator<Item = $data> }; }
super::generate_minister_trait! { IterOps }

/// ## Rust Iterator based Operators
/// Implements a hybrid-push-full operator model.
/// - **Pull** Uses lazily evaluated rust iterators as the stream type
/// - **Push** All single values are structly evaluated, similarly for buffering 
///   operations.
/// 
/// While rust iterators implement a lazily evaluated pull model at a high level. They do not suffer 
/// from the repeated `.next()` calls and option checking in release builds.
/// 
/// In fact, due to in place collection, iterators can be faster than loop. (See the `iterators` benchmark).
/// 
/// ## Interesting Reads
/// - [Comparing Performance: Loops vs Iterators](https://doc.rust-lang.org/book/ch13-04-performance.html)
pub struct Iter;

const ASSUME_SIZE: usize = 1024;
fn get_size(left: Option<usize>, right: Option<usize>) -> usize {
    left.unwrap_or(ASSUME_SIZE) * right.unwrap_or(ASSUME_SIZE)
}
fn get_side_size(hint: Option<usize>) -> usize {
    hint.unwrap_or(ASSUME_SIZE)
}

impl IterOps for Iter {
    fn consume_stream<Data>(iter: impl Iterator<Item = Data>) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        iter
    }

    fn consume_buffer<Data>(buff: Vec<Data>) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        buff.into_iter()
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
        stream
    }

    fn export_buffer<Data>(stream: stream!(Data)) -> Vec<Data>
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
        stream: stream!(Result<Data, Error>),
    ) -> Result<stream!(Data), Error>
    where
        Data: Send + Sync,
        Error: Send + Sync,
    {
        stream.collect::<Result<Vec<_>, _>>().map(Vec::into_iter)
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
        stream.map(mapping)
    }

    fn map_seq<InData, OutData>(
        stream: stream!(InData),
        mapping: impl FnMut(InData) -> OutData,
    ) -> stream!(OutData)
    where
        InData: Send + Sync,
        OutData: Send + Sync,
    {
        stream.map(mapping)
    }

    fn map_single<InData, OutData>(
        single: single!(InData),
        mapping: impl FnOnce(InData) -> OutData,
    ) -> single!(OutData)
    where
        InData: Send + Sync,
        OutData: Send + Sync,
    {
        mapping(single)
    }

    fn filter<Data>(
        stream: stream!(Data),
        predicate: impl Fn(&Data) -> bool + Send + Sync,
    ) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        stream.filter(predicate)
    }

    fn all<Data>(
        stream: stream!(Data),
        predicate: impl Fn(&Data) -> bool + Send + Sync,
    ) -> (bool, stream!(Data))
    where
        Data: Send + Sync,
    {
        let vals = stream.collect::<Vec<_>>();
        (vals.iter().all(predicate), vals.into_iter())
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
        stream.count()
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
        stream.fold(initial, fold_fn)
    }

    fn combine<Data>(
        stream: stream!(Data),
        alternative: Data,
        combiner: impl Fn(Data, Data) -> Data + Send + Sync,
    ) -> single!(Data)
    where
        Data: Send + Sync + Clone,
    {
        stream.reduce(combiner).unwrap_or(alternative)
    }

    fn sort<Data>(
        stream: stream!(Data),
        ordering: impl Fn(&Data, &Data) -> std::cmp::Ordering + Send + Sync,
    ) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        let mut data = stream.collect::<Vec<_>>();
        data.sort_unstable_by(ordering);
        data.into_iter()
    }

    fn take<Data>(stream: stream!(Data), n: usize) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        stream.take(n)
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
        for data in stream {
            let (k, r) = split(data);
            groups.entry(k).or_insert_with(Vec::new).push(r);
        }
        groups.into_iter().map(|(k, v)| (k, v.into_iter()))
    }

    fn cross_join<LeftData, RightData>(
        left: stream!(LeftData),
        right: stream!(RightData),
    ) -> stream!((LeftData, RightData))
    where
        LeftData: Clone + Send + Sync,
        RightData: Clone + Send + Sync,
    {
        let right_vals = right.collect::<Vec<_>>();
        let mut result =
            Vec::with_capacity(right_vals.len() * left.size_hint().1.unwrap_or(ASSUME_SIZE));
        for l in left {
            for r in &right_vals {
                result.push((l.clone(), r.clone()));
            }
        }
        result.into_iter()
    }

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
        match (left.size_hint().1, right.size_hint().1) {
            (Some(left_size), Some(right_size)) if left_size < right_size => {
                let mut results = Vec::with_capacity(left_size * right_size);
                let mut lefts = HashMap::with_capacity(left_size);
                let left = left.collect::<Vec<_>>();
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
                results.into_iter()
            }
            (left_size, right_size) => {
                let mut results = Vec::with_capacity(get_size(left_size, right_size));
                let mut rights = HashMap::with_capacity(get_side_size(right_size));
                let right = right.collect::<Vec<_>>();
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
                results.into_iter()
            }
        }
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
        match (left.size_hint().1, right.size_hint().1) {
            (Some(left_size), Some(right_size)) if left_size < right_size => {
                let left = left.collect::<Vec<_>>();
                let mut results = Vec::with_capacity(left_size * right_size);
                for r in right {
                    for l in &left {
                        if pred(l, &r) {
                            results.push((l.clone(), r.clone()));
                        }
                    }
                }
                results.into_iter()
            }
            (_, right_size) => {
                let right = right.collect::<Vec<_>>();
                let mut results = Vec::with_capacity(get_side_size(right_size));
                for l in left {
                    for r in &right {
                        if pred(&l, r) {
                            results.push((l.clone(), r.clone()));
                        }
                    }
                }
                results.into_iter()
            }
        }
    }

    fn union<Data>(left: stream!(Data), right: stream!(Data)) -> stream!(Data)
    where
        Data: Send + Sync,
    {
        left.chain(right)
    }

    fn fork<Data>(stream: stream!(Data)) -> (stream!(Data), stream!(Data))
    where
        Data: Clone + Send + Sync,
    {
        let data = stream.collect::<Vec<_>>();
        let data2 = data.clone();
        (data.into_iter(), data2.into_iter())
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
        let (left, right): (Vec<_>, Vec<_>) = stream.unzip();
        (left.into_iter(), right.into_iter())
    }
}
