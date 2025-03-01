use std::{
    any::{TypeId, type_name},
    marker::PhantomData,
    sync::{LazyLock, Mutex},
};
use vec_collections::{AbstractVecSet, VecSet};

/// # Safety
/// Implementations must ensure for each type implementing [UniqueToken], only one can exist.
pub unsafe trait UniqueToken {}

struct UniqueClosure<C: FnOnce() + 'static>(PhantomData<C>);

unsafe impl<C: FnOnce() + 'static> UniqueToken for UniqueClosure<C> {}

/// Creates a unique token, with a unique type, that can be used by arenas to
/// eliminate runtime bounds & presence checks.
///  - The input closure is run once the first time it's type is provided.
///
/// ```
/// # use better_arenas::utils::unique2::unique;
/// let unique = unique(||{});
/// ```
///
/// If the type of the closure is used more than once, this will panic.
/// ```should_panic
/// # use better_arenas::utils::unique2::unique;
/// fn same(){}
///
/// let unique_1 = unique(same);
/// let unique_2 = unique(same);
/// ```
///
///
/// Internally usage of types is checked with a [VecSet] of [TypeId], so
/// creating a unique type does have some overhead.
///
/// When using data structures with the unique types, use `impl UniqueToken` for types, and pass
/// the unique token from the toplevel (e.g. main, test case).  
pub fn unique<C: FnOnce() + 'static>(closure: C) -> impl UniqueToken {
    fn unique_inner<C: FnOnce() + 'static>(closure: C) -> UniqueClosure<C> {
        // JUSTIFY: Using a VecSet
        //           - The number of usages is expected to be low, so it is better to use vector.
        //           - Backed by a small-vec, to avoid additional heap allocation for small number
        //             of uniques.
        static USED: LazyLock<Mutex<VecSet<[TypeId; 10]>>> =
            LazyLock::new(|| Mutex::new(VecSet::empty()));

        let mut l = USED.lock().unwrap();
        let id = TypeId::of::<C>();
        if l.contains(&id) {
            panic!(
                "Attempted to use type {} to construct a unique more than once",
                type_name::<C>()
            );
        } else {
            closure();
            // JUSTIFY: We never remove from the set
            //           - Keys using this unique type could exist, even after the unique
            //             object is destroyed
            //           - It is possible to use a similar pattern to the `window pattern` in emdb
            //             to capture a lifetime in the arena, and qualify indicies by that lifetime,
            //             however this complicates the interface
            l.insert(id);
            UniqueClosure(PhantomData)
        }
    }
    unique_inner(closure)
}

/// Creates a unique token unsafely.
///  - Does not check the closure type is already used.
///
/// [unique] does not have zero overhead
///
/// # Safety
/// The user needs to ensure this function is only called once per closure type.
pub unsafe fn unsafe_unique<C: FnOnce() + 'static>(closure: C) -> impl UniqueToken {
    #[cfg(debug_assertions)]
    {
        // JUSTIFY: Use checked implementation when running debug
        //           - Does not affect release, so no release build performance overhead
        unique(closure)
    }

    #[cfg(not(debug_assertions))]
    {
        closure();
        UniqueClosure(PhantomData::<C>)
    }
}
