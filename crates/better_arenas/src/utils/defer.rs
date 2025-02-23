use super::once::Once;

/// A guard that calls the contained function on drop.
pub struct Defer<F: FnOnce()>(Once<F>);

impl<F: FnOnce()> Defer<F> {
    pub fn new(f: F) -> Self {
        Self(Once::new(f))
    }
}

impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            unsafe { self.0.get_once()() }
        }
    }
}

