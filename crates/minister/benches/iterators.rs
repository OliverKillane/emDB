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

use divan;

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn apply(x: usize) -> usize {
    x * 2
}

trait Operate {
    fn op(values: Vec<usize>) -> Vec<usize>;
}

struct Iters;
impl Operate for Iters {
    fn op(values: Vec<usize>) -> Vec<usize> {
        values
            .into_iter()
            .map(apply)
            .map(apply)
            .map(apply)
            .map(apply)
            .collect()
    }
}

struct Loops;
impl Operate for Loops {
    fn op(values: Vec<usize>) -> Vec<usize> {
        let mut result = Vec::with_capacity(values.len());
        for item in values {
            result.push(apply(apply(apply(apply(item)))));
        }
        result
    }
}

#[divan::bench(
    name = "compare_loops_and_iterators",
    types = [Iters, Loops],
    consts = [1, 128, 8388608]
)]
fn comparison<T: Operate, const SIZE: usize>(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| (0..SIZE).collect::<Vec<_>>())
        .bench_local_values(|r| T::op(r));
}

fn main() {
    divan::main()
}
