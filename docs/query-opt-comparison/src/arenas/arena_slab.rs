use super::ArenaCore;
use slab::{Iter, Slab};
struct SlabArena<V>(Slab<V>);

impl<'a, V: 'a> ArenaCore<'a, V> for SlabArena<V> {
    type Index = usize;
    type Iter = Iter<'a, V>;

    fn insert(&self, value: V) -> Self::Index {
        todo!()
    }

    fn get(&self, index: Self::Index) -> Option<&V> {
        todo!()
    }

    fn iter(&self) -> Self::Iter {
        todo!()
    }
}
