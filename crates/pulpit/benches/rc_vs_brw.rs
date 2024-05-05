//! # Benchmarking Reference Counting vs Borrowing
//! The core advantage of the more complex [`pulpit::value`] wrappers is to avoid
//! the need to do reference counting and separate allocations.
//!
//! Here we check the actual cost of reference counting over borrowing for
//! placement and read.

use divan;
use std::{ops::Deref, rc::Rc};
type Contained = [u8; 100];

trait RcRefCmp {
    type Value<'a>: Deref<Target = Contained>
    where
        Self: 'a;

    fn push(&mut self, val: Contained);
    fn new(vals: Vec<Contained>) -> Self;
    fn get(&self, ind: usize) -> Self::Value<'_>;
}

struct Refs {
    data: Vec<Contained>,
}

impl RcRefCmp for Refs {
    type Value<'a> = &'a Contained;

    fn new(vals: Vec<Contained>) -> Self {
        Refs { data: vals }
    }

    fn get(&self, ind: usize) -> Self::Value<'_> {
        &self.data[ind]
    }

    fn push(&mut self, val: Contained) {
        self.data.push(val);
    }
}

struct Rcs {
    data: Vec<Rc<Contained>>,
}

impl RcRefCmp for Rcs {
    type Value<'a> = Rc<Contained>;

    fn new(vals: Vec<Contained>) -> Self {
        Rcs {
            data: vals.into_iter().map(Rc::new).collect(),
        }
    }

    fn get(&self, ind: usize) -> Rc<Contained> {
        self.data[ind].clone()
    }

    fn push(&mut self, val: Contained) {
        self.data.push(Rc::new(val));
    }
}

#[divan::bench(
    name="Scanning values",
    types=[Rcs, Refs],
    consts=[10,100,1000],
)]
fn get_vals<C: RcRefCmp, const SIZE: usize>(bencher: divan::Bencher) {
    let data = (0..SIZE).map(|_| [0; 100]).collect();
    let rcs = C::new(data);
    bencher.bench_local(|| {
        for i in 0..SIZE {
            divan::black_box_drop(rcs.get(i));
        }
    })
}

#[divan::bench(
    name="Pushing values",
    types=[Rcs, Refs],
    consts=[10,100,1000],
)]
fn push_vals<C: RcRefCmp, const SIZE: usize>(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        let mut rcs = C::new(Vec::with_capacity(SIZE));
        for _ in 0..SIZE {
            rcs.push([0; 100]);
        }
        divan::black_box_drop(rcs);
    })
}

fn main() {
    divan::main()
}
