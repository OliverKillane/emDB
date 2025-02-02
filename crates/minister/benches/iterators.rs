//! ## A small example of iterators versus loops in rust.
//! In this example we demonstrate both that rust iterators can be optimised to
//! be as fast as loops (inlined, and converted to iteration).
//!
//! But also due to in-place collection, iterators can be significantly faster than loops.
//! - See [the `in_place_collect.rs` source code](https://github.com/rust-lang/rust/blob/master/library/alloc/src/vec/in_place_collect.rs)
//!
//! When running `cargo bench` you will see that the loop version requires allocating
//! another vector, and deallocating the input vector.
//!
//! No allocations are performed in the iterator version.

// #[global_allocator]
// static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

trait Operate {
    fn op<I, O>(values: Vec<I>, f: impl Fn(I) -> O) -> Vec<O>;
}

struct Iters;
impl Operate for Iters {
    fn op<I, O>(values: Vec<I>, f: impl Fn(I) -> O) -> Vec<O> {
        values.into_iter().map(f).collect()
    }
}

struct Loops;
impl Operate for Loops {
    fn op<I, O>(values: Vec<I>, f: impl Fn(I) -> O) -> Vec<O> {
        let mut result = Vec::with_capacity(values.len());
        for item in values {
            result.push(f(item));
        }
        result
    }
}

#[divan::bench(
    name = "Iterators versus Loops",
    types = [Iters, Loops],
    consts = [2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288]
)]
fn comparison<T: Operate, const SIZE: usize>(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| (0..SIZE).map(|i| (i, i)).collect::<Vec<_>>())
        .bench_local_values(|r| T::op(r, |(x, y)| x + y));
}

fn main() {
    divan::main()
}
