//! # Minister
//! A library for implementing stream operators. Used by `emDB` for the physical
//! implementation of operators.
//!
//! > **Note**
//! > The [parallel] and [chunk] implementations are not optimised & should not be used.
//! > [iter] is the best performing.

pub mod basic;
pub mod chunk;
pub mod iter;
pub mod parallel;
pub mod morsel;

/// ## Minister Trait Generation
/// In order to ensure correct implementation of different operator implementations (important for
/// emDB's code generation), a single interface for Operators is needed.
///
/// The requirements are as follows:
/// 1. A simplified set of operators that are composable for implementing higher level emDB operations.
/// 2. To allow operators to execute on data in parallel
/// 3. To allow operators to define their own types for streams and single values.
///
/// Ordinarily this would be satisifed by a single `trait Operator {}`, however the
/// requirement to allow implementations to define their own stream types includes defining streams
/// as any implementation of a trait.
///
/// For example, for the [iter::Iter] backend uses streams as `impl Iterator<item=Data>`.
///
/// To implement as a trait would require being able to define an associated item, that is either a
/// type, or a trait.
/// - This work is at RFC stage as part of the [Impl trait Initiative](https://rust-lang.github.io/impl-trait-initiative/)
///
/// Hence instead we generate a trait, substituting the types using other macros (`single!` and `stream!`).
/// - The `single!` and `stream!` macros need to be defined in the same scope as the trait.
/// ```
/// # trait Thunk { type Item; }
/// # trait ThunkIterator { type Item; }
/// # use minister::generate_minister_trait;
/// macro_rules! single { ($data:ty) => { impl Thunk<Item=$data> }; }
/// macro_rules! stream { ($data:ty) => { impl ThunkIterator<Item=$data> }; }
/// generate_minister_trait! { LazyOps }
/// ```
///
/// ## Operator Types
/// While the operator pattern supported appears to be push based, pull based operators can also be
/// supported by pushing a lazily evaluated stream.
/// - While [basic::Basic] is a traditional pull-based operator, [iter::Iter] is sort-of-pull based (with
///   some pipeline breakage for expanding errors, and notably the ability of the rust compiler to
///   combine/inline the operations from a pull).
/// - A fully lazy 'iterators of thunks' implementation is also possible with this pattern.
///
/// The push-like pattern makes code generation significantly easier, especially when emDB supports
/// plans that are DAGs (operators can pull data from and push to any number of sources).
/// - This is also discussed by [snowflake](https://info.snowflake.net/rs/252-RFO-227/images/Snowflake_SIGMOD.pdf)
///   as an advantage.
///
/// ### Interesting Reads
/// - [Justin Jaffray: Push vs Pull](https://justinjaffray.com/query-engines-push-vs.-pull/)
/// - [snowflake paper](https://info.snowflake.net/rs/252-RFO-227/images/Snowflake_SIGMOD.pdf)
/// - [Push vs Pull-Based Loop Fusion in Query Engines](https://arxiv.org/pdf/1610.09166)
#[macro_export]
macro_rules! generate_minister_trait {
    ($trait_name:ident) => {
        pub trait $trait_name {
            fn consume_stream<Data>(iter: impl Iterator<Item = Data>) -> stream!(Data)
            where
                Data: Send + Sync;
            fn consume_buffer<Data>(buff: Vec<Data>) -> stream!(Data)
            where
                Data: Send + Sync;
            fn consume_single<Data>(data: Data) -> single!(Data)
            where
                Data: Send + Sync;

            fn export_stream<Data>(stream: stream!(Data)) -> impl Iterator<Item = Data>
            where
                Data: Send + Sync;
            fn export_buffer<Data>(stream: stream!(Data)) -> Vec<Data>
            where
                Data: Send + Sync;
            fn export_single<Data>(single: single!(Data)) -> Data
            where
                Data: Send + Sync;

            fn error_stream<Data, Error>(
                stream: stream!(Result<Data, Error>),
            ) -> Result<stream!(Data), Error>
            where
                Data: Send + Sync,
                Error: Send + Sync;

            fn error_single<Data, Error>(
                single: single!(Result<Data, Error>),
            ) -> Result<single!(Data), Error>
            where
                Data: Send + Sync,
                Error: Send + Sync;

            fn map<InData, OutData>(
                stream: stream!(InData),
                mapping: impl Fn(InData) -> OutData + Send + Sync,
            ) -> stream!(OutData)
            where
                InData: Send + Sync,
                OutData: Send + Sync;

            fn map_seq<InData, OutData>(
                stream: stream!(InData),
                mapping: impl FnMut(InData) -> OutData,
            ) -> stream!(OutData)
            where
                InData: Send + Sync,
                OutData: Send + Sync;

            fn map_single<InData, OutData>(
                single: single!(InData),
                mapping: impl FnOnce(InData) -> OutData,
            ) -> single!(OutData)
            where
                InData: Send + Sync,
                OutData: Send + Sync;

            fn filter<Data>(
                stream: stream!(Data),
                predicate: impl Fn(&Data) -> bool + Send + Sync,
            ) -> stream!(Data)
            where
                Data: Send + Sync;

            fn all<Data>(
                stream: stream!(Data),
                predicate: impl Fn(&Data) -> bool + Send + Sync,
            ) -> (bool, stream!(Data))
            where
                Data: Send + Sync;

            fn is<Data>(
                single: single!(Data),
                predicate: impl Fn(&Data) -> bool,
            ) -> (bool, single!(Data))
            where
                Data: Send + Sync;

            fn count<Data>(stream: stream!(Data)) -> single!(usize)
            where
                Data: Send + Sync;

            fn fold<InData, Acc>(
                stream: stream!(InData),
                initial: Acc,
                fold_fn: impl Fn(Acc, InData) -> Acc,
            ) -> single!(Acc)
            where
                InData: Send + Sync,
                Acc: Send + Sync;

            fn combine<Data>(
                stream: stream!(Data),
                alternative: Data,
                combiner: impl Fn(Data, Data) -> Data + Send + Sync,
            ) -> single!(Data)
            where
                Data: Send + Sync + Clone;

            fn sort<Data>(
                stream: stream!(Data),
                ordering: impl Fn(&Data, &Data) -> std::cmp::Ordering + Send + Sync,
            ) -> stream!(Data)
            where
                Data: Send + Sync;

            fn take<Data>(stream: stream!(Data), n: usize) -> stream!(Data)
            where
                Data: Send + Sync;

            fn group_by<Key, Rest, Data>(
                stream: stream!(Data),
                split: impl Fn(Data) -> (Key, Rest),
            ) -> stream!((Key, stream!(Rest)))
            where
                Data: Send + Sync,
                Key: Eq + std::hash::Hash + Send + Sync,
                Rest: Send + Sync;

            fn cross_join<LeftData, RightData>(
                left: stream!(LeftData),
                right: stream!(RightData),
            ) -> stream!((LeftData, RightData))
            where
                LeftData: Clone + Send + Sync,
                RightData: Clone + Send + Sync;

            fn equi_join<LeftData, RightData, Key>(
                left: stream!(LeftData),
                right: stream!(RightData),
                left_split: impl Fn(&LeftData) -> &Key + Send + Sync,
                right_split: impl Fn(&RightData) -> &Key + Send + Sync,
            ) -> stream!((LeftData, RightData))
            where
                Key: Eq + std::hash::Hash + Send + Sync,
                LeftData: Clone + Send + Sync,
                RightData: Clone + Send + Sync;

            fn predicate_join<LeftData, RightData>(
                left: stream!(LeftData),
                right: stream!(RightData),
                pred: impl Fn(&LeftData, &RightData) -> bool + Send + Sync,
            ) -> stream!((LeftData, RightData))
            where
                LeftData: Clone + Send + Sync,
                RightData: Clone + Send + Sync;

            fn union<Data>(left: stream!(Data), right: stream!(Data)) -> stream!(Data)
            where
                Data: Send + Sync;

            fn fork<Data>(stream: stream!(Data)) -> (stream!(Data), stream!(Data))
            where
                Data: Clone + Send + Sync;

            fn fork_single<Data>(single: single!(Data)) -> (single!(Data), single!(Data))
            where
                Data: Clone + Send + Sync;

            fn split<LeftData, RightData>(
                stream: stream!((LeftData, RightData)),
            ) -> (stream!(LeftData), stream!(RightData))
            where
                LeftData: Send + Sync,
                RightData: Send + Sync;
        }
    };
}
