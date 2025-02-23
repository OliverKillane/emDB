use function_name::named;
use std::mem::MaybeUninit;

pub struct Once<T> {
    data: MaybeUninit<T>,

    #[cfg(debug_assertions)]
    removed: bool,
}

impl<T> Once<T> {
    pub fn new(t: T) -> Self {
        Self {
            data: MaybeUninit::new(t),
            removed: false,
        }
    }

    #[named]
    pub unsafe fn get_once(&mut self) -> T {
        #[cfg(debug_assertions)]
        {
            let fn_name = function_name!();
            assert!(!self.removed, "{fn_name} called more than once");
            self.removed = true
        }
        unsafe { self.data.assume_init_read() }
    }
}
