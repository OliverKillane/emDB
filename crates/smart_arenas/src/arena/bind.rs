//! # Binding arenas to additional allocators
//! Used for decomposed storage, the arena generates keys, the allocator generates the rest.

// keys as part of the arena
// taking the unique type
// only instantiated once
// KeyGuard

// has a trait including size Key<Unique, Size>
// Key<Unique, Size>::get_guard() -> the thing
// Then pass to the arena
// The use -> data can contain the key type

use std::mem::MaybeUninit;

use crate::{
    alloc::{AllocImpl, AllocSelect},
    key::{KeyTrait, WeakKey},
};

use super::{Arena, CopyKeyArena, DeleteArena, WriteArena};

pub struct Bind<AData, Primary: Arena, Assoc: AllocSelect> {
    primary: Primary,
    assoc: Assoc::Impl<<Primary::Key as KeyTrait>::Idx, MaybeUninit<AData>>,
}

impl<AData, Primary: Arena, Assoc: AllocSelect> Arena for Bind<AData, Primary, Assoc> {
    type Key = Primary::Key;
    type Data = (Primary::Data, AData);
    type Read<'a>
        = (Primary::Read<'a>, &'a AData)
    where
        Self: 'a;

    fn new(preallocate_to: <Self::Key as KeyTrait>::Idx) -> Self {
        Self {
            primary: Primary::new(preallocate_to),
            assoc: Assoc::Impl::new(preallocate_to),
        }
    }

    fn insert_return_reuse(&mut self, (pdata, adata): Self::Data) -> Option<(Self::Key, bool)> {
        let key_reuse = self.primary.insert_return_reuse(pdata);
        if let Some((key, reuse)) = &key_reuse {
            if *reuse {
                unsafe {
                    let slot = self.assoc.write(key.to_idx());
                    slot.write(adata);
                }
            } else {
                self.assoc.append(MaybeUninit::new(adata));
            }
        }
        key_reuse
    }

    fn read(&self, key: &Self::Key) -> Self::Read<'_> {
        (self.primary.read(key), unsafe {
            &self.assoc.read(key.to_idx()).assume_init_ref()
        })
    }

    fn len(&self) -> usize {
        self.primary.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (WeakKey<'a, Self::Key>, Self::Read<'a>)> + 'a {
        self.primary.iter().map(|(wk, pdata)| {
            let adata = unsafe { self.assoc.read(wk.brw().to_idx()).assume_init_ref() };
            (wk, (pdata, adata))
        })
    }
}

impl<AData, Primary: DeleteArena, Assoc: AllocSelect> DeleteArena for Bind<AData, Primary, Assoc> {
    fn delete_return_dropped(&mut self, key: Self::Key) -> bool {
        let idx = key.to_idx();
        let dropped = self.primary.delete_return_dropped(key);
        if dropped {
            unsafe {
                self.assoc.write(idx).assume_init_drop();
            }
        }
        dropped
    }
}

impl<AData, Primary: WriteArena, Assoc: AllocSelect> WriteArena for Bind<AData, Primary, Assoc> {
    type Write<'a>
        = (Primary::Write<'a>, &'a mut AData)
    where
        Self: 'a;

    fn write(&mut self, key: &Self::Key) -> Self::Write<'_> {
        let idx = key.to_idx();
        let pdata = self.primary.write(key);
        let adata = unsafe { self.assoc.write(idx).assume_init_mut() };
        (pdata, adata)
    }
}

impl<AData, Primary: CopyKeyArena, Assoc: AllocSelect> CopyKeyArena
    for Bind<AData, Primary, Assoc>
{
    fn copy_key(&mut self, key: &<Self as Arena>::Key) -> Option<<Self as Arena>::Key> {
        self.primary.copy_key(key)
    }
}
