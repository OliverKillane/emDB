use minister::{
    basic::{Basic, BasicOps},
    chunk::{Chunk, ChunkOps},
    iter::{Iter, IterOps},
    parallel::{Parallel, ParallelOps},
};

/// As each operator implementation uses a separate generated trait, we need to
/// unify as a single type for the benchmarks, we do that by implementing this trait.
trait Microbench {
    fn groupby<Key, Rest, Data>(
        stream: Vec<Data>,
        split: impl Fn(Data) -> (Key, Rest),
    ) -> Vec<(Key, Vec<Rest>)>
    where
        Data: Send + Sync,
        Key: Eq + std::hash::Hash + Send + Sync,
        Rest: Send + Sync;

    fn chain_2_map<DataIn, DataMed, DataOut>(
        buffer: Vec<DataIn>,
        mapping1: impl Fn(DataIn) -> DataMed + Send + Sync,
        mapping2: impl Fn(DataMed) -> DataOut + Send + Sync,
    ) -> Vec<DataOut>
    where
        DataIn: Send + Sync,
        DataOut: Send + Sync,
        DataMed: Send + Sync;

    /// For benchmarking the impact of combining maps together.
    fn chain_1_map<DataIn, DataOut>(
        buffer: Vec<DataIn>,
        mapping: impl Fn(DataIn) -> DataOut + Send + Sync,
    ) -> Vec<DataOut>
    where
        DataIn: Send + Sync,
        DataOut: Send + Sync;

        fn equijoin<LeftData, RightData, Key>(
            left: Vec<LeftData>,
            right: Vec<RightData>,
            left_split: impl Fn(&LeftData) -> &Key + Send + Sync,
            right_split: impl Fn(&RightData) -> &Key + Send + Sync,
        ) -> Vec<(LeftData, RightData)>
        where
            Key: Eq + std::hash::Hash + Send + Sync,
            LeftData: Clone + Send + Sync,
            RightData: Clone + Send + Sync;
}

macro_rules! impl_microbench {
    ($($name:ident),*) => {
        $( impl Microbench for $name {
            fn groupby<Key, Rest, Data>(
                buffer: Vec<Data>,
                split: impl Fn(Data) -> (Key, Rest),
            ) -> Vec<(Key, Vec<Rest>)>
            where
                Data: Send + Sync,
                Key: Eq + std::hash::Hash + Send + Sync,
                Rest: Send + Sync {
                    let stream = $name::consume_buffer(buffer);
                    let grouping = $name::group_by(stream, split);
                    let collected = $name::map(grouping, |(k, v)| (k, $name::export_buffer(v)));
                    $name::export_buffer(collected)
                }


            fn chain_2_map<DataIn, DataMed, DataOut>(
                buffer: Vec<DataIn>,
                mapping_1: impl Fn(DataIn) -> DataMed + Send + Sync,
                mapping_2: impl Fn(DataMed) -> DataOut + Send + Sync,
            ) -> Vec<DataOut>
            where
                DataIn: Send + Sync,
                DataOut: Send + Sync,
                DataMed: Send + Sync
                 {
                    let stream = $name::consume_buffer(buffer);
                    let mapped_1 = $name::map(stream, mapping_1);
                    let mapped_2 = $name::map(mapped_1, mapping_2);
                    $name::export_buffer(mapped_2)
                }

            fn chain_1_map<DataIn, DataOut>(
                buffer: Vec<DataIn>,
                mapping: impl Fn(DataIn) -> DataOut + Send + Sync,
            ) -> Vec<DataOut>
            where
                DataIn: Send + Sync, DataOut: Send + Sync {
                    let stream = $name::consume_buffer(buffer);
                    let mapped = $name::map(stream, mapping);
                    $name::export_buffer(mapped)
                }

            fn equijoin<LeftData, RightData, Key>(
                left: Vec<LeftData>,
                right: Vec<RightData>,
                left_split: impl Fn(&LeftData) -> &Key + Send + Sync,
                right_split: impl Fn(&RightData) -> &Key + Send + Sync,
            ) -> Vec<(LeftData, RightData)>
            where
                Key: Eq + std::hash::Hash + Send + Sync,
                LeftData: Clone + Send + Sync,
                RightData: Clone + Send + Sync {
                    let left_stream = $name::consume_buffer(left);
                    let right_stream = $name::consume_buffer(right);
                    let joined = $name::equi_join(left_stream, right_stream, left_split, right_split);
                    $name::export_buffer(joined)
                }
        })*
    }

}

impl_microbench!(Basic, Iter, Parallel, Chunk);

const SCALE_FACTORS: [usize; 5] = [65536, 131072, 262144, 524288, 1048576];

fn new_buffer<V, const SCALE_FACTOR: usize>(f: impl Fn(usize) -> V) -> Vec<V> {
    use rand::prelude::SliceRandom;
    let mut buff = (0..SCALE_FACTOR).collect::<Vec<_>>();
    buff.shuffle(&mut rand::thread_rng());
    buff.into_iter().map(f).collect()
}

#[divan::bench(
    name = "GroupBy on integers",
    types = [Basic, Iter, Parallel, Chunk],
    consts = SCALE_FACTORS,
)]
fn groupby<T: Microbench, const SCALE_FACTOR: usize>(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| new_buffer::<(usize, usize), SCALE_FACTOR>(|k| (k % 10, k)))
        .bench_local_values(|r| T::groupby(r, |(k, v)| (k, v)));
}


#[divan::bench(
    name = "A single map operation",
    types = [Basic, Iter, Parallel, Chunk],
    consts = SCALE_FACTORS,
)]
fn chain_1_map<T: Microbench, const SCALE_FACTOR: usize>(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| new_buffer::<usize, SCALE_FACTOR>(|k| k))
        .bench_local_values(|r| T::chain_1_map(r, |a| a + 1));
}

/// Provides an advantage to implementations that avoid intermediate buffers.
#[divan::bench(
    name = "Two map operations chained together",
    types = [Basic, Iter, Parallel, Chunk],
    consts = SCALE_FACTORS,
)]
fn chain_2_map<T: Microbench, const SCALE_FACTOR: usize>(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| new_buffer::<usize, SCALE_FACTOR>(|k| k))
        .bench_local_values(|r| T::chain_2_map(r, |a| a + 1, |a| a + 1));
}

/// TODO: parameterize the difference in size, and the number of join partners.
#[divan::bench(
    name = "Join two streams of integers",
    types = [Basic, Iter, Parallel, Chunk],
    consts = [32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384],
)]
fn equijoin<T: Microbench, const SCALE_FACTOR: usize>(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| (new_buffer::<usize, SCALE_FACTOR>(|k| k), new_buffer::<usize, SCALE_FACTOR>(|k| k)))
        .bench_local_values(|(left, right)| T::equijoin(left, right, |a| a, |b| b));
}

fn main() {
    divan::main()
}
