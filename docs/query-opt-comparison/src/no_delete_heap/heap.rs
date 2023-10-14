//! A basic fixed size heap implementation to demonstrate the efficiency gains
//! possible from the no_delete_heap implementation

/// A datastructure to support a fixed number of items in a heap.
struct FixedHeap<const N: usize, T, F>
where
    T: Clone + Copy,
    F: Fn(&T, &T) -> bool,
{
    data: [Option<T>; N],
    size: usize,
    cmp: F,
}

impl<const N: usize, T, F> FixedHeap<N, T, F>
where
    T: Clone + Copy,
    F: Fn(&T, &T) -> bool,
{
    fn new(cmp: F) -> Self {
        Self {
            data: [None; N],
            size: 0,
            cmp,
        }
    }

    fn insert(&mut self, val: T) {}

    fn get_front(&self) -> Option<&T> {
        None
    }
}
