/// A trait for unique marker types.
///  - Types that can only be used once, cannot be shared.
///  - Allows for different declarations of runtime values
pub trait Unique {
    fn runtime_check(self);
}
impl<T: FnOnce()> Unique for T {
    fn runtime_check(self) {
        self()
    }
}

/// Creates a unique marker type.
///  - Internally uses a closure that cannot be run (compile time panic if call reachable).
#[macro_export]
macro_rules! unique {
    () => {
        {
            fn unique_gen() -> impl FnOnce() {
                || {
                    use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
                    static UNIQUE: AtomicBool = AtomicBool::new(false);
                    if let Err(_) = UNIQUE.compare_exchange(false, true, Relaxed, Relaxed) {
                        panic!("Attempted to use unique type more than once");
                    }
                }
            }
            unique_gen()
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_values() {
        let u = unique!();
        u();
    }

    #[test]
    #[should_panic]
    fn duplicated_unique() {
        fn create_unique() -> impl Unique {
            unique!()
        }

        let x = create_unique();
        let y = create_unique();

        x.runtime_check();
        y.runtime_check();
    }
}