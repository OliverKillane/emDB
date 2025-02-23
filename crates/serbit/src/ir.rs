use std::mem::{self, ManuallyDrop, MaybeUninit};

trait DropWith {
    type Arg;
    fn drop_with(self, arg: Self::Arg);
}

struct DropWithWrapper<D: DropWith>(MaybeUninit<D>);

impl<D: DropWith> DropWithWrapper<D> {
    pub fn new(data: D) -> Self {
        Self(MaybeUninit::new(data))
    }

    fn drop_with(self, arg: <D as DropWith>::Arg) {
        unsafe {
            // JUSTIFY: copy inner into `drop_with` destructor
            //           - We use mem::forget to prevent drop being called, so cannot call twice.
            self.0.assume_init_read().drop_with(arg);
        }
        mem::forget(self);
    }
}

impl<D: DropWith> Drop for DropWithWrapper<D> {
    fn drop(&mut self) {
        const { panic!("Attempted to drop undroppable type") }
    }
}

trait RcArena<Data> {
    type Index;

    fn new() -> Self;

    fn brw(&self, idx: &Self::Index) -> &Data;
    fn brw_mut(&self, idx: &Self::Index) -> &mut Data;
    fn insert(&mut self, data: Data) -> Self::Index;

    fn inc(&self, idx: &Self::Index) -> Self::Index;
    fn dec(&self, idx: Self::Index);
}
