use assume::assume;

use super::*;

use std::{
    mem::{size_of, ManuallyDrop},
    ptr,
};

// TODO: Bench against 0 as missing
/// The next free splot to reuse.
/// Note: Cannot be `usize::Max`
struct NextFree(Option<usize>);
type EncodedNextFree = usize;

impl NextFree {
    #[inline(always)]
    fn encode(&self) -> usize {
        if let Some(index) = self.0 {
            assume!(unsafe: index != EncodedNextFree::MAX, "index is invalid" );
            index
        } else {
            EncodedNextFree::MAX
        }
    }

    #[inline(always)]
    fn decode(val: EncodedNextFree) -> Self {
        NextFree(if val == EncodedNextFree::MAX {
            None
        } else {
            Some(val)
        })
    }
}

struct HiddenData<MutData> {
    hidden: bool,
    data: MutData,
}

union Slot<MutData> {
    full: ManuallyDrop<HiddenData<MutData>>,
    next_free: EncodedNextFree,
}

struct MutEntry<ImmData, MutData> {
    imm_ptr: *const ImmData,
    mut_data: Slot<MutData>,
}

impl<ImmData, MutData> Drop for MutEntry<ImmData, MutData> {
    fn drop(&mut self) {
        if self.imm_ptr.is_null() {
            unsafe {
                ManuallyDrop::drop(&mut self.mut_data.full);
            }
        }
    }
}

/// A generational arena that retains immutable data to allow for immutable,
/// stable references to be taken.
///
/// # Leaks
/// This arena *retains* immutable data until the arena is dropped, as a result
/// it can accumulate large amounts of immutable values.
/// - Detremental for large, frequently deleted and inserted tables on machines
///   with limited memory.
/// - Not a true leak (i.e. like [`std::mem::forget`]), data is still cleared on
///   drop. Though if a table is retained for the entire program run, this makes
///   no difference.
///
/// # Generations
/// The immutable data pointer is used as the generation counter.
/// - No extra space overhead & need this pointer anyway when accessing [PrimaryWindow::get].
/// - As each new allocation for a non-zero sized object is unique, this gives
///   us a value to use for generation.
/// - For zero size types we have the same
///
/// This strategy does not work for zero sized types, so in this instance, we
/// use the immutable data pointer's location as a normal generation counter,
/// and pass [transmute]-ed references to an internal zero-sized type out.
///
/// ```
/// # use std::mem::{MaybeUninit, size_of};
/// assert_eq!(size_of::<()>(), 0);
/// assert_eq!(size_of::<[MaybeUninit<()>; 10]>(), 0);
/// let x: [MaybeUninit<()>; 10] = [MaybeUninit::new(()); 10];
/// unsafe {
///     assert_eq!(x[0].as_ptr(), x[9].as_ptr());
/// }
/// ```  
pub struct PrimaryRetain<ImmData, MutData, const BLOCK_SIZE: usize> {
    mut_data: Vec<MutEntry<ImmData, MutData>>,
    visible_count: usize,
    next_free_mut: NextFree,
    imm_data: utils::Blocks<ImmData, BLOCK_SIZE>,
    gen_counter: usize,
    dummy_zero_size: (),
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Keyable
    for PrimaryRetain<ImmData, MutData, BLOCK_SIZE>
{
    type Key = GenKey<PrimaryRetain<ImmData, MutData, BLOCK_SIZE>, *const ImmData>;
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Column
    for PrimaryRetain<ImmData, MutData, BLOCK_SIZE>
{
    type WindowKind<'imm> = Window<'imm, PrimaryRetain<ImmData, MutData, BLOCK_SIZE>>
    where
        Self: 'imm;

    fn new(size_hint: usize) -> Self {
        PrimaryRetain {
            mut_data: Vec::with_capacity(size_hint / BLOCK_SIZE + 1),
            imm_data: utils::Blocks::new(size_hint),
            visible_count: 0,
            next_free_mut: NextFree(None),
            gen_counter: 1,
            dummy_zero_size: (),
        }
    }

    fn window(&mut self) -> Self::WindowKind<'_> {
        Window { inner: self }
    }
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> PrimaryWindow<'imm, ImmData, MutData>
    for Window<'imm, PrimaryRetain<ImmData, MutData, BLOCK_SIZE>>
where
    MutData: Clone,
    ImmData: Clone,
{
    type ImmGet = &'imm ImmData;
    type Col = PrimaryRetain<ImmData, MutData, BLOCK_SIZE>;

    #[inline(always)]
    fn get(&self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmGet, MutData> {
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

    #[inline(always)]
    fn brw(&self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &MutData> {
        if let Some(MutEntry { imm_ptr, mut_data }) = self.inner.mut_data.get(key.index) {
            unsafe {
                if key.generation == *imm_ptr && !mut_data.full.hidden {
                    Ok(Entry {
                        index: key.index,
                        data: Data {
                            imm_data: if size_of::<ImmData>() == 0 {
                                transmute::<&(), &ImmData>(&self.inner.dummy_zero_size)
                            } else {
                                &*(*imm_ptr).cast::<ImmData>()
                            },
                            mut_data: &mut_data.full.data,
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

    #[inline(always)]
    fn brw_mut(&mut self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &mut MutData> {
        if let Some(MutEntry { imm_ptr, mut_data }) = self.inner.mut_data.get_mut(key.index) {
            unsafe {
                if key.generation == *imm_ptr && !mut_data.full.hidden {
                    Ok(Entry {
                        index: key.index,
                        data: Data {
                            imm_data: if size_of::<ImmData>() == 0 {
                                transmute::<&(), &ImmData>(&self.inner.dummy_zero_size)
                            } else {
                                &*(*imm_ptr).cast::<ImmData>()
                            },
                            mut_data: &mut mut_data.full.data,
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

    #[inline(always)]
    fn conv_get(get: Self::ImmGet) -> ImmData {
        get.clone()
    }

    #[inline(always)]
    fn scan<'brw>(&'brw self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + 'brw {
        self.inner
            .mut_data
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if entry.imm_ptr.is_null() {
                    None
                } else {
                    Some(GenKey {
                        index,
                        generation: entry.imm_ptr,
                        phantom: PhantomData,
                    })
                }
            })
    }

    #[inline(always)]
    fn count(&self) -> usize {
        self.inner.visible_count
    }
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> PrimaryWindowPull<'imm, ImmData, MutData>
    for Window<'imm, PrimaryRetain<ImmData, MutData, BLOCK_SIZE>>
where
    MutData: Clone,
    ImmData: Clone,
{
    type ImmPull = &'imm ImmData;

    #[inline(always)]
    fn insert(
        &mut self,
        Data { imm_data, mut_data }: Data<ImmData, MutData>,
    ) -> (<Self::Col as Keyable>::Key, InsertAction) {
        let imm_ptr = self.inner.imm_data.append(imm_data);
        self.inner.visible_count += 1;
        if let NextFree(Some(next_free)) = self.inner.next_free_mut {
            unsafe {
                let mut_entry = self.inner.mut_data.get_unchecked_mut(next_free);
                debug_assert!(mut_entry.imm_ptr.is_null());

                let imm_ptr = if size_of::<ImmData>() == 0 {
                    // For zero sized types, use the generation counter.
                    let val = self.inner.gen_counter as *const ImmData;
                    self.inner.gen_counter += 1;
                    val
                } else {
                    imm_ptr
                };

                self.inner.next_free_mut = NextFree::decode(mut_entry.mut_data.next_free);
                *mut_entry = MutEntry {
                    imm_ptr,
                    mut_data: Slot {
                        full: ManuallyDrop::new(HiddenData {
                            hidden: false,
                            data: mut_data,
                        }),
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
                    full: ManuallyDrop::new(HiddenData {
                        hidden: false,
                        data: mut_data,
                    }),
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

    #[inline(always)]
    fn pull(&mut self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmPull, MutData> {
        if let Some(mut_entry) = self.inner.mut_data.get_mut(key.index) {
            unsafe {
                if key.generation == mut_entry.imm_ptr {
                    let pull_imm_ref = if size_of::<ImmData>() == 0 {
                        transmute::<&(), &ImmData>(&self.inner.dummy_zero_size)
                    } else {
                        &*(mut_entry.imm_ptr).cast::<ImmData>()
                    };
                    let pull_mut_data = ManuallyDrop::take(&mut mut_entry.mut_data.full);
                    if !pull_mut_data.hidden {
                        self.inner.visible_count -= 1;
                    }
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
                            mut_data: pull_mut_data.data,
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

    fn conv_pull(pull: Self::ImmPull) -> ImmData {
        pull.clone()
    }
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> PrimaryWindowHide<'imm, ImmData, MutData>
    for Window<'imm, PrimaryRetain<ImmData, MutData, BLOCK_SIZE>>
where
    MutData: Clone,
    ImmData: Clone,
{
    #[inline(always)]
    fn hide(&mut self, key: <Self::Col as Keyable>::Key) -> Result<(), KeyError> {
        if let Some(MutEntry { imm_ptr, mut_data }) = self.inner.mut_data.get_mut(key.index) {
            unsafe {
                if key.generation == *imm_ptr && !mut_data.full.hidden {
                    mut_data.full.hidden = true;
                    self.inner.visible_count -= 1;
                    Ok(())
                } else {
                    Err(KeyError)
                }
            }
        } else {
            Err(KeyError)
        }
    }

    #[inline(always)]
    fn reveal(&mut self, key: <Self::Col as Keyable>::Key) -> Result<(), KeyError> {
        if let Some(MutEntry { imm_ptr, mut_data }) = self.inner.mut_data.get_mut(key.index) {
            unsafe {
                if key.generation == *imm_ptr && mut_data.full.hidden {
                    mut_data.full.hidden = false;
                    self.inner.visible_count += 1;
                    Ok(())
                } else {
                    Err(KeyError)
                }
            }
        } else {
            Err(KeyError)
        }
    }
}

#[cfg(kani)]
impl<ImmData, MutData, const BLOCK_SIZE: usize> kani::Arbitrary
    for GenKey<PrimaryRetain<ImmData, MutData, BLOCK_SIZE>, *const ImmData>
{
    fn any() -> Self {
        let mut gen_ind: usize = kani::any();
        if gen_ind == 0 {
            gen_ind += 1;
        }
        Self {
            index: kani::any(),
            generation: (gen_ind as *const ImmData),
            phantom: PhantomData,
        }
    }
}
