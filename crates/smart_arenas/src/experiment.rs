//! ## Unique Ids with lifetimes
//! Inspired by ghost cell, we can remove the type map lookup, use an arbitrary number of lifetimes & hence arenas.
//!  - Makes testable & easier to use / less boilerplate
//!
//! However, the error messages are bad.
//!  - Not easy to read or clear, but complete
//!  - Currently requires nightly rust, but we can see about how to remove this
//!  
//! I think the tradeoff is worth it.

use std::marker::{PhantomData, PhantomInvariantLifetime};

use crate::key::IdxInt;

trait KeyTrait<'id> {}

struct Key<'id, Idx: IdxInt> {
    idx: Idx,
    _phantom: PhantomInvariantLifetime<'id>,
}

impl<'id, Idx: IdxInt> KeyTrait<'id> for Key<'id, Idx> {}

struct Token<'id>(PhantomInvariantLifetime<'id>);

trait Arena<'id> {
    type Key: KeyTrait<'id>;

    fn new(tk: Token<'id>) -> Self;
    fn get_key(&self) -> Self::Key;
    fn use_key(&self, k: &Self::Key);
}

// higher kind lifetime
fn make_tk<R, F: for<'id> FnOnce(Token<'id>) -> R>(cl: F) -> R {
    cl(Token(PhantomInvariantLifetime::new()))
}

struct ConcArena<'id, Idx: IdxInt> {
    t: Token<'id>,
    _phantom: PhantomData<Idx>,
}

impl<'id, Idx: IdxInt> Arena<'id> for ConcArena<'id, Idx> {
    type Key = Key<'id, Idx>;

    fn new(tk: Token<'id>) -> Self {
        todo!()
    }

    fn get_key(&self) -> Self::Key {
        todo!()
    }

    fn use_key(&self, k: &Self::Key) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_basic() {
        make_tk(|tk1| {
            make_tk(|tk2| {
                let arena1 = ConcArena::<'_, u32>::new(tk1);
                let arena2 = ConcArena::<'_, u32>::new(tk2);

                let key1 = arena1.get_key();
                let key2 = arena2.get_key();

                arena1.use_key(&key1); // Fails with &key2
                arena2.use_key(&key2);
            })
        });
    }

    #[test]
    fn check_function() {
        fn access<'a, 'b, K: KeyTrait<'a>>(
            x: &impl Arena<'a, Key = K>,
            y: &impl Arena<'b>, // Fails in the arena creation if we use Key = K
            k: &K,
        ) {
            x.use_key(k);
            // y.use_key(k); // Causes failure in key type
        }
        make_tk(|tk1| {
            make_tk(|tk2| {
                let arena1 = ConcArena::<'_, u32>::new(tk1);
                let arena2 = ConcArena::<'_, u32>::new(tk2);

                let key1 = arena1.get_key();
                let key2 = arena2.get_key();

                access(&arena1, &arena2, &key1);
                // access(&arena1, &arena2, &key2); // Causes arena construction to fail
            })
        })
    }

    #[test]
    fn check_returns() {
        make_tk(|tk1| {
            // ConcArena::<'_, u32>::new(tk1) // Does not live long enough / cannot be returned

            let arena1 = ConcArena::<'_, u32>::new(tk1);
            // arena1.get_key() // Cannot return (key does not live long enough)
        });
    }
}
