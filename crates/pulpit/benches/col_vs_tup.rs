//! Compare access performance from separate vectors versus a single vector.
use divan;
trait Access<A, B>
where
    A: Clone,
    B: Clone,
{
    fn new(size_hint: usize) -> Self;
    fn get_all(&self, ind: usize) -> Option<(&A, &B)>;
    fn get_a(&self, ind: usize) -> Option<&A>;
    fn get_b(&self, ind: usize) -> Option<&B>;
    fn put(&mut self, ind: usize, val: (A, B)) -> bool;
    fn append(&mut self, val: (A, B)) -> usize;
}

struct Decomp<A, B> {
    a_data: Vec<A>,
    b_data: Vec<B>,
}

impl<A, B> Access<A, B> for Decomp<A, B>
where
    A: Clone,
    B: Clone,
{
    fn new(size_hint: usize) -> Self {
        Self {
            a_data: Vec::with_capacity(size_hint),
            b_data: Vec::with_capacity(size_hint),
        }
    }

    fn get_all(&self, ind: usize) -> Option<(&A, &B)> {
        if let Some(a) = self.a_data.get(ind) {
            unsafe {
                let b = self.b_data.get_unchecked(ind);
                Some((a, b))
            }
        } else {
            None
        }
    }

    fn put(&mut self, ind: usize, (a, b): (A, B)) -> bool {
        if let Some(a_r) = self.a_data.get_mut(ind) {
            unsafe {
                let b_r = self.b_data.get_unchecked_mut(ind);
                *a_r = a;
                *b_r = b;
                true
            }
        } else {
            false
        }
    }

    fn append(&mut self, (a, b): (A, B)) -> usize {
        let next_ind = self.a_data.len();
        self.a_data.push(a);
        self.b_data.push(b);
        next_ind
    }

    fn get_a(&self, ind: usize) -> Option<&A> {
        self.a_data.get(ind)
    }

    fn get_b(&self, ind: usize) -> Option<&B> {
        self.b_data.get(ind)
    }
}

struct Tuple<A, B> {
    data: Vec<(A, B)>,
}

impl<A, B> Access<A, B> for Tuple<A, B>
where
    A: Clone,
    B: Clone,
{
    fn new(size_hint: usize) -> Self {
        Self {
            data: Vec::with_capacity(size_hint),
        }
    }

    fn get_all(&self, ind: usize) -> Option<(&A, &B)> {
        self.data.get(ind).map(|(a, b)| (a, b))
    }

    fn put(&mut self, ind: usize, val: (A, B)) -> bool {
        if let Some(d) = self.data.get_mut(ind) {
            *d = val;
            true
        } else {
            false
        }
    }

    fn append(&mut self, val: (A, B)) -> usize {
        let ind = self.data.len();
        self.data.push(val);
        ind
    }

    fn get_a(&self, ind: usize) -> Option<&A> {
        self.data.get(ind).map(|(a, _)| a)
    }

    fn get_b(&self, ind: usize) -> Option<&B> {
        self.data.get(ind).map(|(_, b)| b)
    }
}

#[divan::bench(
    name="Comparing push performance of tuple vs decomp",
    types=[Tuple<usize,usize>, Decomp<usize,usize>],
)]
fn push_vals<V: Access<usize, usize>>(bencher: divan::Bencher) {
    const ITERS: usize = 100000;
    bencher.bench_local(|| {
        let mut v = V::new(ITERS);
        for i in 0..ITERS {
            v.append((i, i));
        }
        divan::black_box_drop(v)
    })
}

#[divan::bench(
    name="Comparing get all performance of tuple vs decomp",
    types=[Tuple<usize,usize>, Decomp<usize,usize>],
)]
fn get_vals<V: Access<usize, usize>>(bencher: divan::Bencher) {
    const ITERS: usize = 100000;
    let mut v = V::new(ITERS);
    for i in 0..ITERS {
        v.append((i, i));
    }
    bencher.bench_local(|| {
        for i in 0..ITERS {
            divan::black_box_drop(v.get_all(i));
        }
    });
    divan::black_box_drop(v)
}

#[divan::bench(
    name="Comparing get just a column performance of tuple vs decomp",
    types=[Tuple<usize,usize>, Decomp<usize,usize>],
)]
fn get_a_vals<V: Access<usize, usize>>(bencher: divan::Bencher) {
    const ITERS: usize = 10000000;
    let mut v = V::new(ITERS);
    for i in 0..ITERS {
        v.append((i, i));
    }
    bencher.bench_local(|| {
        for i in 0..ITERS {
            divan::black_box_drop(v.get_a(i));
        }
    });
    divan::black_box_drop(v)
}

fn main() {
    divan::main()
}
