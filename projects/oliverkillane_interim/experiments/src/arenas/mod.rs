//! A set of benchmarks to compare key-value stores of different types and capabilities.

pub mod arena_gen;
pub mod arena_slab;
pub mod arena_std;

/// Basic methods required for any arena
trait ArenaCore<'a, V: 'a> {
    type Index;
    type Iter: Iterator<Item = (Self::Index, &'a V)>;

    fn insert(&self, value: V) -> Self::Index;
    fn get(&self, index: Self::Index) -> Option<&V>;
    fn iter(&self) -> Self::Iter;
}

/// Some arenas allow update
trait ArenaUpdate<'a, V: 'a>: ArenaCore<'a, V> {
    fn get_mut(&self, index: Self::Index) -> Option<&mut V>;
}

/// Some arenas allow deletion
trait ArenaDelete<'a, V: 'a>: ArenaCore<'a, V> {
    fn remove(&self, index: Self::Index) -> bool;
}
