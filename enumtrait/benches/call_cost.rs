#[enumtrait::store(black_hole_trait)]
trait CallBlackHole {
    fn call(&mut self);
}
pub(crate) use black_hole_trait;

// must be separate as new takes no params, this works fine for enumtrait, but not for dyn trait objects
trait BuildAble {
    fn new() -> Self;
}

// Implement the enumtraits
macro_rules! new_impl {
    ($mod_name:ident use $data:ident => $first:ident , $($i:ident),*) => {
        mod $mod_name {
            use super::{CallBlackHole, BuildAble, black_hole_trait};

            pub struct $first;
            impl CallBlackHole for $first {
                fn call(&mut self) { divan::black_box(self); }
            }
            impl BuildAble for $first {
                fn new() -> Self { Self }
            }

            $(
                pub struct $i;
                impl CallBlackHole for $i {
                    fn call(&mut self) { divan::black_box(self); }
                }
                impl BuildAble for $i {
                    fn new() -> Self { Self }
                }
            )*

            #[enumtrait::quick_enum]
            #[enumtrait::store(enum_tks)]
            pub enum $data {
                $first,
                $($i,)*
            }

            impl BuildAble for $data {
                fn new() -> Self { $data::$first($first::new()) }
            }

            #[enumtrait::impl_trait(black_hole_trait for enum_tks)]
            impl CallBlackHole for $data {
            }
        }
        use $mod_name::$data; 
    }
}

new_impl!(single  use Single  => A,);
new_impl!(double  use Double  => A,B);
new_impl!(sixteen use Sixteen => A,B,C,D,E,F,G,H,I,J,K,L,M,N,O);

// implementing using dyn
struct DynInner;
impl CallBlackHole for DynInner {
    fn call(&mut self) { divan::black_box(self); }
}
impl BuildAble for DynInner {
    fn new() -> Self { Self }
}
struct ImplDyn(Box<dyn CallBlackHole>);
impl CallBlackHole for ImplDyn {
    fn call(&mut self) { self.0.call(); }
}
impl BuildAble for ImplDyn {
    fn new() -> Self { Self(Box::new(DynInner::new())) }
}

// implementing as a single concrete type for baseline
struct Concrete;
impl CallBlackHole for Concrete {
    fn call(&mut self) { divan::black_box(self); }
}
impl BuildAble for Concrete {
    fn new() -> Self { Self }
}

/// Benchmarking the cost of function calls
/// - Allocation is not included in the benchmark
/// - Only blackhole used (want to check call only)
/// - Repeated calls to the same member (perfect for caching)
const CALL_TOTALS: [usize; 3] = [1, 16, 268435456];
#[divan::bench(
    name = "call_with_blackhole",
    types = [Concrete, ImplDyn, Single, Double, Sixteen],
    consts = CALL_TOTALS
)]
fn bench<B: CallBlackHole + BuildAble, const CALLS: usize>(bencher: divan::Bencher) {
    let mut callee = B::new();
    bencher.bench_local(|| {
        for _ in 0..CALLS {
            callee.call();
        }
    });
}

fn main() {
    divan::main();
}
