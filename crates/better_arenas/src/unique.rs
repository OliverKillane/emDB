
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
    () => { ||{
        static UNIQUE: AtomicBool = AtomicBool::new(false);
        if !UNIQUE.compare_exchange(false, true, Ordering::Relaxed) {
            panic!("Attempted to use unique type more than once");
        }
    } };
}