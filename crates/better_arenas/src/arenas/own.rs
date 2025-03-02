use super::{
    Arena, DeleteArena, Store, WriteArena,
    common::{self, Key, ValOrFree},
};
use crate::{
    alloc::{AllocImpl, AllocSelect},
    utils::{idx::IdxInt, unique::UniqueToken},
};
use std::{marker::PhantomData, mem::ManuallyDrop};

/// An arena providing keys that own their values ([Box], but for an arena).
///  - Keys cannot be dropped, they must be deleted (otherwise they panic at runtime)
///  - Keys cannot be copied, and must be moved. No aliasing of keys is possible.
///
/// The implementation internally contains a singly-linked list of free allocations &
/// reuses allocations that are deleted.
///
/// This arena safely eliminates all checks on access, [Own::read] and [Own::write].
///  - Simply an allocator lookup using an index.
///  - Depending on the allocator, this can be reduced to an unchecked array access.
///
/// Access across arenas with the same data type is not possible, keys from a given
/// arena can only be used on the same arena, enforced by [UniqueToken].
///  - Attempting to use the same type for a token more than once panics at runtime.
///
/// ```
/// use better_arenas::prelude::*;
///
/// #[derive(Clone, PartialEq, Eq, Debug)]
/// struct ExampleData {
///     foo: u32,
///     bar: [u8; 4],
/// }
///
/// let mut example_arena = Own::<u8, Plain<ExampleData>, Contig, _>::new(OwnConfig{
///     unique: unique(||{}),
///     alloc: ContigConfig{ preallocate_to: 100 }
/// });
///
/// let example_data = ExampleData{ foo: 42, bar: [1, 2, 3, 4] };
///
/// if let Some(key) = example_arena.insert(example_data.clone()) {
///     let data = example_arena.read(&key);
///     assert_eq!(example_data, *data);
///     
///     // Must delete the key.
///     example_arena.delete(key);
/// } else {
///     // Have reached the maximum number of allocations supported
///     //  - Either the arena, allocator beneath cannot allocate (e.g. because the next index is
///     //    greater than `u8` can support)
/// }
/// ```
///
/// We get a compile time error if we try to use indexes from a different arena.
/// ```compile_fail,E0308
/// # use better_arenas::prelude::*;
/// # #[derive(Clone, PartialEq, Eq, Debug)]
/// # struct ExampleData {
/// #     foo: u32,
/// #     bar: [u8; 4],
/// # }
/// let mut arena_1 = Own::<u8, Plain<ExampleData>, Contig, _>::new(OwnConfig{
///     unique: unique(||{}),
///     alloc: ContigConfig{ preallocate_to: 100 }
/// });
/// let mut arena_2 = Own::<u8, Plain<ExampleData>, Contig, _>::new(OwnConfig{
///     unique: unique(||{}),
///     alloc: ContigConfig{ preallocate_to: 100 }
/// });
///
/// let key_1 = arena_1.insert(ExampleData{ foo: 42, bar: [1, 2, 3, 4] }).expect("Insert to empty own arena should always succeed");
///
/// arena_2.read(&key_1);
/// ```
pub struct Own<Idx: IdxInt, S: Store, Alloc: AllocSelect, Id: UniqueToken> {
    // JUSTIFY: Complex type
    //          Splitting this into a type alias would not make it simpler.
    #[allow(clippy::type_complexity)]
    slots: Alloc::Impl<Idx, ValOrFree<Idx, S::Data<<Own<Idx, S, Alloc, Id> as Arena<S>>::Key>>>,
    next_free: Option<Idx>,
    len: usize,
    _phantom: PhantomData<Id>,
}

pub struct OwnConfig<AllocCfg, Id: UniqueToken> {
    pub unique: Id,
    pub alloc: AllocCfg,
}

impl<Idx: IdxInt, S: Store, Alloc: AllocSelect, Id: UniqueToken> Arena<S>
    for Own<Idx, S, Alloc, Id>
{
    type Cfg = OwnConfig<
        <Alloc::Impl<Idx, ValOrFree<Idx, S::Data<Self::Key>>> as AllocImpl<
            Idx,
            ValOrFree<Idx, S::Data<Self::Key>>,
        >>::Cfg,
        Id,
    >;
    type Key = Key<Idx, Id>;

    fn new(Self::Cfg { unique: _, alloc }: Self::Cfg) -> Self {
        Self {
            slots: Alloc::Impl::new(alloc),
            next_free: None,
            len: 0,
            _phantom: PhantomData,
        }
    }

    fn insert(&mut self, data: S::Data<Self::Key>) -> Option<Self::Key> {
        if let Some(idx) = self.next_free {
            unsafe {
                let slot = self.slots.write(idx);
                self.next_free = *slot.next_free;
                ManuallyDrop::drop(&mut slot.next_free);
                slot.data = ManuallyDrop::new(data);
            }
            self.len += 1;
            Some(Key::new(idx))
        } else if let Some(idx) = self.slots.insert(ValOrFree {
            data: ManuallyDrop::new(data),
        }) {
            self.len += 1;
            Some(Key::new(idx))
        } else {
            None
        }
    }

    fn read(&self, key: &Self::Key) -> &S::Data<Self::Key> {
        // JUSTIFY: No bounds check on lookup of the key.
        //           - Keys can only be created from this module
        //           - Keys cannot be copied
        //           - Keys include a unique type marker, checked at construction.
        //          Hence it is only possible use a key, if it has been provided by this specific
        //          instance.
        // JUSTIFY: No check on union.
        //           - Keys cannot be copied, and deletion takes ownership of a key
        //          Hence this key must have been from an insert, and cannot have been deleted.
        unsafe { &self.slots.read(key.0.idx).data }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a <S as Store>::Data<Self::Key>> + 'a
    where
        <S as Store>::Data<<Self as Arena<S>>::Key>: 'a,
    {
        common::Iter::new(&self.slots, self.next_free, self.len())
    }
}

impl<Idx: IdxInt, S: Store, Alloc: AllocSelect, Id: UniqueToken> DeleteArena<S>
    for Own<Idx, S, Alloc, Id>
{
    fn delete(&mut self, key: Self::Key) {
        unsafe {
            let value = self.slots.write(key.0.idx);
            ManuallyDrop::drop(&mut value.data);
            value.next_free = ManuallyDrop::new(self.next_free);
        }
        self.next_free = Some(key.0.idx);
        key.0.drop(());
        self.len -= 1;
    }
}

impl<Idx: IdxInt, S: Store, Alloc: AllocSelect, Id: UniqueToken> WriteArena<S>
    for Own<Idx, S, Alloc, Id>
{
    fn write(&mut self, key: &Self::Key) -> &mut S::Data<Self::Key> {
        // JUSTIFY: No bounds check on lookup of the key.
        //           - Keys can only be created from this module
        //           - Keys cannot be copied
        //           - Keys include a unique type marker, checked at construction.
        //          Hence it is only possible use a key, if it has been provided by this specific
        //          instance.
        // JUSTIFY: No check on union.
        //           - Keys cannot be copied, and deletion takes ownership of a key
        //          Hence this key must have been from an insert, and cannot have been deleted.
        unsafe { &mut self.slots.write(key.0.idx).data }
    }
}
