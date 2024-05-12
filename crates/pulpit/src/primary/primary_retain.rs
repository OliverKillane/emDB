use super::*;

use std::{mem::ManuallyDrop, ptr};

// TODO: Bench against 0 as missing
/// The next free splot to reuse.
/// Note: Cannot be `usize::Max`
struct NextFree(Option<usize>);
type EncodedNextFree = usize;

impl NextFree {
    fn encode(&self) -> usize {
        if let Some(index) = self.0 {
            debug_assert!(index != EncodedNextFree::MAX);
            return index;
        } else {
            EncodedNextFree::MAX
        }
    }

    fn decode(val: EncodedNextFree) -> Self {
        NextFree(if val == EncodedNextFree::MAX {
            None
        } else {
            Some(val)
        })
    }
}

union Slot<MutData> {
    data: ManuallyDrop<MutData>,
    next_free: EncodedNextFree,
}

struct MutEntry<ImmData, MutData> {
    imm_ptr: *const ImmData,
    mut_data: Slot<MutData>,
}

impl<ImmData, MutData> Drop for MutEntry<ImmData, MutData> {
    fn drop(&mut self) {
        if self.imm_ptr != ptr::null() {
            unsafe {
                ManuallyDrop::drop(&mut self.mut_data.data);
            }
        }
    }
}

pub struct ColRet<ImmData, MutData, const BLOCK_SIZE: usize> {
    mut_data: Vec<MutEntry<ImmData, MutData>>,
    next_free_mut: NextFree,
    imm_data: utils::Blocks<ImmData, BLOCK_SIZE>,
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Store for ColRet<ImmData, MutData, BLOCK_SIZE> {
    type WindowKind<'imm> = Window<'imm, ColRet<ImmData, MutData, BLOCK_SIZE>>
    where
        Self: 'imm;

    fn new(size_hint: usize) -> Self {
        ColRet {
            mut_data: Vec::with_capacity(size_hint / BLOCK_SIZE + 1),
            imm_data: utils::Blocks::new(size_hint),
            next_free_mut: NextFree(None),
        }
    }

    fn window<'imm>(&'imm mut self) -> Self::WindowKind<'imm> {
        Window { inner: self }
    }
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> PrimaryWindow<'imm, ImmData, MutData>
    for Window<'imm, ColRet<ImmData, MutData, BLOCK_SIZE>>
where
    MutData: Clone,
{
    type ImmGet = &'imm ImmData;
    type Key = GenKey<ColRet<ImmData, MutData, BLOCK_SIZE>, *const ImmData>;

    fn get(&self, key: Self::Key) -> Access<Self::ImmGet, MutData> {
        let Entry {
            index,
            data: Data { imm_data, mut_data },
        } = self.brw(key)?;
        Ok(Entry {
            index,
            data: Data {
                imm_data: unsafe { transmute::<&ImmData, &'imm ImmData>(imm_data) },
                mut_data: mut_data.clone(),
            },
        })
    }

    fn brw(&self, key: Self::Key) -> Access<&ImmData, &MutData> {
        if let Some(MutEntry { imm_ptr, mut_data }) = self.inner.mut_data.get(key.index) {
            if key.generation == *imm_ptr {
                Ok(Entry {
                    index: key.index,
                    data: Data {
                        imm_data: unsafe { &*(*imm_ptr).cast::<ImmData>() },
                        mut_data: unsafe { &mut_data.data },
                    },
                })
            } else {
                Err(KeyError)
            }
        } else {
            Err(KeyError)
        }
    }

    fn brw_mut(&mut self, key: Self::Key) -> Access<&ImmData, &mut MutData> {
        if let Some(MutEntry { imm_ptr, mut_data }) = self.inner.mut_data.get_mut(key.index) {
            unsafe {
                if key.generation == *imm_ptr {
                    Ok(Entry {
                        index: key.index,
                        data: Data {
                            imm_data: &*(*imm_ptr).cast::<ImmData>(),
                            mut_data: &mut mut_data.data,
                        },
                    })
                } else {
                    Err(KeyError)
                }
            }
        } else {
            Err(KeyError)
        }
    }
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> PrimaryWindowPull<'imm, ImmData, MutData>
    for Window<'imm, ColRet<ImmData, MutData, BLOCK_SIZE>>
where
    MutData: Clone,
{
    type ImmPull = &'imm ImmData;

    fn insert(
        &mut self,
        Data { imm_data, mut_data }: Data<ImmData, MutData>,
    ) -> (Self::Key, InsertAction) {
        let imm_ptr = self.inner.imm_data.append(imm_data);
        if let NextFree(Some(next_free)) = self.inner.next_free_mut {
            unsafe {
                let mut_entry = self.inner.mut_data.get_unchecked_mut(next_free);
                debug_assert!(mut_entry.imm_ptr == ptr::null());
                self.inner.next_free_mut = NextFree::decode(mut_entry.mut_data.next_free);
                *mut_entry = MutEntry {
                    imm_ptr,
                    mut_data: Slot {
                        data: ManuallyDrop::new(mut_data),
                    },
                };
                (
                    GenKey {
                        index: next_free,
                        generation: imm_ptr,
                        phantom: PhantomData,
                    },
                    InsertAction::Place(next_free),
                )
            }
        } else {
            let index = self.inner.mut_data.len();
            self.inner.mut_data.push(MutEntry {
                imm_ptr,
                mut_data: Slot {
                    data: ManuallyDrop::new(mut_data),
                },
            });
            (
                GenKey {
                    index,
                    generation: imm_ptr,
                    phantom: PhantomData,
                },
                InsertAction::Append,
            )
        }
    }

    fn pull(&mut self, key: Self::Key) -> Access<Self::ImmPull, MutData> {
        if let Some(mut_entry) = self.inner.mut_data.get_mut(key.index) {
            unsafe {
                if key.generation == mut_entry.imm_ptr {
                    let pull_imm_ref = &*(mut_entry.imm_ptr).cast::<ImmData>();
                    let pull_mut_data = ManuallyDrop::take(&mut mut_entry.mut_data.data);
                    *mut_entry = MutEntry {
                        imm_ptr: ptr::null(),
                        mut_data: Slot {
                            next_free: self.inner.next_free_mut.encode(),
                        },
                    };
                    self.inner.next_free_mut = NextFree(Some(key.index));
                    Ok(Entry {
                        index: key.index,
                        data: Data {
                            imm_data: pull_imm_ref,
                            mut_data: pull_mut_data,
                        },
                    })
                } else {
                    Err(KeyError)
                }
            }
        } else {
            Err(KeyError)
        }
    }
}
