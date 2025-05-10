use std::mem::MaybeUninit;

use crate::{
    alloc::{AllocImpl, AllocSelect},
    id::{
        key::{KeyTrait, WeakKey},
        token::Token,
    },
};

use super::{Arena, CopyKeyArena, DeleteArena, WriteArena};

/// ## Columnar Storage for Arenas / 'Struct of Arrays'
/// An arena combinator that allows associating additional data in separate allocators with an arena type.
///  - The arena type determines the smart behaviour (e.g [crate::arena::Own] or [crate::arena::Share])
///  - No additional bounds/access checks are required for the associated column.
pub struct Bind<'id, AData, Primary: Arena<'id>, Assoc: AllocSelect> {
    primary: Primary,
    assoc: Assoc::Impl<<Primary::Key as KeyTrait<'id>>::Idx, MaybeUninit<AData>>,
}

impl<'id, AData, Primary: Arena<'id>, Assoc: AllocSelect> Arena<'id>
    for Bind<'id, AData, Primary, Assoc>
{
    type Key = Primary::Key;
    type Data = (Primary::Data, AData);
    type Read<'a>
        = (Primary::Read<'a>, &'a AData)
    where
        Self: 'a;

    fn new(preallocate_to: <Self::Key as KeyTrait<'id>>::Idx, token: Token<'id>) -> Self {
        Self {
            primary: Primary::new(preallocate_to, token),
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
            self.assoc.read(key.to_idx()).assume_init_ref()
        })
    }

    fn len(&self) -> usize {
        self.primary.len()
    }

    fn iter_with_weak_key<'a>(
        &'a self,
    ) -> impl Iterator<Item = (WeakKey<'id, 'a, Self::Key>, Self::Read<'a>)> + 'a {
        self.primary.iter_with_weak_key().map(|(wk, pdata)| {
            let adata = unsafe { self.assoc.read(wk.to_idx()).assume_init_ref() };
            (wk, (pdata, adata))
        })
    }
}

impl<'id, AData, Primary: DeleteArena<'id>, Assoc: AllocSelect> DeleteArena<'id>
    for Bind<'id, AData, Primary, Assoc>
{
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

impl<'id, AData, Primary: WriteArena<'id>, Assoc: AllocSelect> WriteArena<'id>
    for Bind<'id, AData, Primary, Assoc>
{
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

impl<'id, AData, Primary: CopyKeyArena<'id>, Assoc: AllocSelect> CopyKeyArena<'id>
    for Bind<'id, AData, Primary, Assoc>
{
    fn copy_key(&mut self, key: &<Self as Arena<'id>>::Key) -> Option<<Self as Arena<'id>>::Key> {
        self.primary.copy_key(key)
    }
}
