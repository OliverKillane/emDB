//! ## Morsel-Driven Parallelism Operators
//!
//! Designed for parallelism, still using [`Iterator`]s for volcano processing
//! of data.
//!
//! ## Potential Improvement
//! ### Statistics interface
//! each operator should be provided its own statistics store, which can then be
//! used to optimise splitting, cardinality estimates, selectivity estimates, etc.
//!
//! ### Timing Estimates
//! Use system timing to sample the complexity of operations, for use in
//! statistics.

mod buffer;
mod datum;
mod morsel;
mod splitter;
mod statistics;

macro_rules! single {
    ($data:ty) => {
        $data
    };
}

macro_rules! stream { ($data:ty) => { impl morsel::Morsel<Data = $data> }; }

super::generate_minister_trait! { MorselOps }

pub struct Morsel;

// impl MorselOps for Morsel {
//     fn consume_stream<Data>(iter:impl Iterator<Item = Data>) -> stream!(Data)where Data:Send+Sync {
//         morsel::Read::new(iter.collect())
//     }

//     fn consume_buffer<Data>(buff:Vec<Data>) -> stream!(Data)where Data:Send+Sync {
//         todo!()
//     }

//     fn consume_single<Data>(data:Data) -> single!(Data)where Data:Send+Sync {
//         todo!()
//     }

//     fn export_stream<Data>(stream:stream!(Data)) -> impl Iterator<Item = Data>where Data:Send+Sync {
//         todo!()
//     }

//     fn export_buffer<Data>(stream:stream!(Data)) -> Vec<Data>where Data:Send+Sync {
//         todo!()
//     }

//     fn export_single<Data>(single:single!(Data)) -> Data where Data:Send+Sync {
//         todo!()
//     }

//     fn error_stream<Data,Error>(stream:stream!(Result<Data,Error>),) -> Result<stream!(Data),Error>where Data:Send+Sync,Error:Send+Sync {
//         todo!()
//     }

//     fn error_single<Data,Error>(single:single!(Result<Data,Error>),) -> Result<single!(Data),Error>where Data:Send+Sync,Error:Send+Sync {
//         todo!()
//     }

//     fn map<InData,OutData>(stream:stream!(InData),mapping:impl Fn(InData) -> OutData+Send+Sync,) -> stream!(OutData)where InData:Send+Sync,OutData:Send+Sync {
//         todo!()
//     }

//     fn map_seq<InData,OutData>(stream:stream!(InData),mapping:impl FnMut(InData) -> OutData,) -> stream!(OutData)where InData:Send+Sync,OutData:Send+Sync {
//         todo!()
//     }

//     fn map_single<InData,OutData>(single:single!(InData),mapping:impl FnOnce(InData) -> OutData,) -> single!(OutData)where InData:Send+Sync,OutData:Send+Sync {
//         todo!()
//     }

//     fn filter<Data>(stream:stream!(Data),predicate:impl Fn(&Data) -> bool+Send+Sync,) -> stream!(Data)where Data:Send+Sync {
//         todo!()
//     }

//     fn all<Data>(stream:stream!(Data),predicate:impl Fn(&Data) -> bool+Send+Sync,) -> (bool,stream!(Data))where Data:Send+Sync {
//         todo!()
//     }

//     fn is<Data>(single:single!(Data),predicate:impl Fn(&Data) -> bool,) -> (bool,single!(Data))where Data:Send+Sync {
//         todo!()
//     }

//     fn count<Data>(stream:stream!(Data)) -> single!(usize)where Data:Send+Sync {
//         todo!()
//     }

//     fn fold<InData,Acc>(stream:stream!(InData),initial:Acc,fold_fn:impl Fn(Acc,InData) -> Acc,) -> single!(Acc)where InData:Send+Sync,Acc:Send+Sync {
//         todo!()
//     }

//     fn combine<Data>(stream:stream!(Data),alternative:Data,combiner:impl Fn(Data,Data) -> Data+Send+Sync,) -> single!(Data)where Data:Send+Sync+Clone {
//         todo!()
//     }

//     fn sort<Data>(stream:stream!(Data),ordering:impl Fn(&Data, &Data) -> std::cmp::Ordering+Send+Sync,) -> stream!(Data)where Data:Send+Sync {
//         todo!()
//     }

//     fn take<Data>(stream:stream!(Data),n:usize) -> stream!(Data)where Data:Send+Sync {
//         todo!()
//     }

//     fn group_by<Key,Rest,Data>(stream:stream!(Data),split:impl Fn(Data) -> (Key,Rest),) -> stream!((Key,stream!(Rest)))where Data:Send+Sync,Key:Eq+std::hash::Hash+Send+Sync,Rest:Send+Sync {
//         todo!()
//     }

//     fn cross_join<LeftData,RightData>(left:stream!(LeftData),right:stream!(RightData),) -> stream!((LeftData,RightData))where LeftData:Clone+Send+Sync,RightData:Clone+Send+Sync {
//         todo!()
//     }

//     fn equi_join<LeftData,RightData,Key>(left:stream!(LeftData),right:stream!(RightData),left_split:impl Fn(&LeftData) ->  &Key+Send+Sync,right_split:impl Fn(&RightData) ->  &Key+Send+Sync,) -> stream!((LeftData,RightData))where Key:Eq+std::hash::Hash+Send+Sync,LeftData:Clone+Send+Sync,RightData:Clone+Send+Sync {
//         todo!()
//     }

//     fn predicate_join<LeftData,RightData>(left:stream!(LeftData),right:stream!(RightData),pred:impl Fn(&LeftData, &RightData) -> bool+Send+Sync,) -> stream!((LeftData,RightData))where LeftData:Clone+Send+Sync,RightData:Clone+Send+Sync {
//         todo!()
//     }

//     fn union<Data>(left:stream!(Data),right:stream!(Data)) -> stream!(Data)where Data:Send+Sync {
//         todo!()
//     }

//     fn fork<Data>(stream:stream!(Data)) -> (stream!(Data),stream!(Data))where Data:Clone+Send+Sync {
//         todo!()
//     }

//     fn fork_single<Data>(single:single!(Data)) -> (single!(Data),single!(Data))where Data:Clone+Send+Sync {
//         todo!()
//     }

//     fn split<LeftData,RightData>(stream:stream!((LeftData,RightData)),) -> (stream!(LeftData),stream!(RightData))where LeftData:Send+Sync,RightData:Send+Sync {
//         todo!()
//     }
//     // ..
// }
