//! # Minister
//! A library for implementing stream operators, including against pulpit tables.

mod basic;
pub use basic::Basic;

/// A trait for implementing a set of physical operators
/// - Operator implementations can take advantage of the stream implementation
/// - Operators do not have a concept of error.
pub trait Physical {
    type Stream<Data>;
    type Single<Data>: Into<Data> + From<Data>;

    fn consume_stream<Data>(iter: impl Iterator<Item = Data>) -> Self::Stream<Data>;
    fn consume_single<Data>(data: Data) -> Self::Single<Data>;

    fn export_stream<Data>(stream: Self::Stream<Data>) -> impl Iterator<Item = Data>;
    fn export_single<Data>(single: Self::Single<Data>) -> Data;

    fn error_stream<Data, Error>(
        stream: Self::Stream<Result<Data, Error>>,
    ) -> Result<Self::Stream<Data>, Error>;

    fn error_single<Data, Error>(
        stream: Self::Single<Result<Data, Error>>,
    ) -> Result<Self::Single<Data>, Error>;

    fn map<InData, OutData>(
        stream: Self::Stream<InData>,
        mapping: impl Fn(InData) -> OutData,
    ) -> Self::Stream<OutData>;

    fn map_seq<InData, OutData>(
        stream: Self::Stream<InData>,
        mapping: impl FnMut(InData) -> OutData,
    ) -> Self::Stream<OutData>;

    fn map_single<InData, OutData>(
        single: Self::Single<InData>,
        mapping: impl FnOnce(InData) -> OutData,
    ) -> Self::Single<OutData>;

    fn filter<Data>(
        stream: Self::Stream<Data>,
        predicate: impl Fn(&Data) -> bool,
    ) -> Self::Stream<Data>;

    fn fold<InData, Acc>(
        stream: Self::Stream<InData>,
        initial: Acc,
        fold_fn: impl Fn(&mut Acc, InData),
    ) -> Self::Single<Acc>;

    fn combine<Data>(
        stream: Self::Stream<Data>,
        combiner: impl Fn(Data, Data) -> Data,
    ) -> Self::Single<Data>;

    fn sort<Data>(
        stream: Self::Stream<Data>,
        ordering: impl FnMut(&Data, &Data) -> std::cmp::Ordering,
    ) -> Self::Stream<Data>;

    fn take<Data>(stream: Self::Stream<Data>, n: usize) -> Self::Stream<Data>;

    fn groupby<Key, Rest>(
        stream: Self::Stream<(Key, Rest)>,
    ) -> Self::Stream<(Key, Self::Stream<Rest>)>
    where
        Key: Eq + std::hash::Hash;

    fn cross_join<LeftData, RightData>(
        left: Self::Stream<LeftData>,
        right: Self::Stream<RightData>,
    ) -> Self::Stream<(LeftData, RightData)>
    where
        LeftData: Clone,
        RightData: Clone;

    fn equi_join<LeftData, RightData, Key>(
        left: Self::Stream<LeftData>,
        right: Self::Stream<RightData>,
        left_split: impl Fn(&LeftData) -> &Key,
        right_split: impl Fn(&RightData) -> &Key,
    ) -> Self::Stream<(LeftData, RightData)>
    where
        Key: Eq + std::hash::Hash,
        LeftData: Clone,
        RightData: Clone;

    fn predicate_join<LeftData, RightData>(
        left: Self::Stream<LeftData>,
        right: Self::Stream<RightData>,
        pred: impl Fn(&LeftData, &RightData) -> bool,
    ) -> Self::Stream<(LeftData, RightData)>
    where
        LeftData: Clone,
        RightData: Clone;

    fn union<Data>(left: Self::Stream<Data>, right: Self::Stream<Data>) -> Self::Stream<Data>;

    fn fork<Data, const SPLIT: usize>(stream: Self::Stream<Data>) -> [Self::Stream<Data>; SPLIT]
    where
        Data: Clone;

    fn split<LeftData, RightData>(
        stream: Self::Stream<(LeftData, RightData)>,
    ) -> (Self::Stream<LeftData>, Self::Stream<RightData>);
}
