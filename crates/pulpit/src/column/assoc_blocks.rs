use super::*;
// TODO: optimise by keeping end pointer immediately available

/// An associated, append only [`Column`] that stores mutable and immutable data together in
/// blocks, and provides stable references to the immutable part.
pub struct AssocBlocks<ImmData, MutData, const BLOCK_SIZE: usize> {
    blocks: utils::Blocks<Data<ImmData, MutData>, BLOCK_SIZE>,
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Column
    for AssocBlocks<ImmData, MutData, BLOCK_SIZE>
{
    type WindowKind<'imm> = Window<'imm, AssocBlocks<ImmData, MutData, BLOCK_SIZE>>
    where
        Self: 'imm;

    fn new(size_hint: usize) -> Self {
        AssocBlocks {
            blocks: utils::Blocks::new(size_hint),
        }
    }

    fn window(&mut self) -> Self::WindowKind<'_> {
        Window { inner: self }
    }
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> AssocWindow<'imm, ImmData, MutData>
    for Window<'imm, AssocBlocks<ImmData, MutData, BLOCK_SIZE>>
where
    MutData: Clone,
    ImmData: Clone,
{
    type ImmGet = &'imm ImmData;

    #[inline(always)]
    unsafe fn assoc_get(&self, ind: UnsafeIndex) -> Data<Self::ImmGet, MutData> {
        unsafe {
            let Data { imm_data, mut_data } =
                <Self as AssocWindow<'imm, ImmData, MutData>>::assoc_brw(self, ind);
            Data {
                imm_data: transmute::<&ImmData, &'imm ImmData>(imm_data),
                mut_data: mut_data.clone(),
            }
        }
    }

    #[inline(always)]
    unsafe fn assoc_brw(&self, ind: UnsafeIndex) -> Data<&ImmData, &MutData> {
        unsafe {
            let Data { imm_data, mut_data } = self.inner.blocks.get(ind);
            Data { imm_data, mut_data }
        }
    }

    #[inline(always)]
    unsafe fn assoc_brw_mut(&mut self, ind: UnsafeIndex) -> Data<&ImmData, &mut MutData> {
        unsafe {
            let Data { imm_data, mut_data } = self.inner.blocks.get_mut(ind);
            Data { imm_data, mut_data }
        }
    }

    #[inline(always)]
    fn assoc_append(&mut self, val: Data<ImmData, MutData>) {
        self.inner.blocks.append(val);
    }

    #[inline(always)]
    fn conv_get(get: Self::ImmGet) -> ImmData {
        get.clone()
    }

    #[inline(always)]
    unsafe fn assoc_unppend(&mut self) {
        self.inner.blocks.unppend();
    }
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Keyable
    for AssocBlocks<ImmData, MutData, BLOCK_SIZE>
{
    type Key = UnsafeIndex;
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> PrimaryWindow<'imm, ImmData, MutData>
    for Window<'imm, AssocBlocks<ImmData, MutData, BLOCK_SIZE>>
where
    MutData: Clone,
    ImmData: Clone,
{
    type ImmGet = &'imm ImmData;
    type Col = AssocBlocks<ImmData, MutData, BLOCK_SIZE>;

    #[inline(always)]
    fn get(&self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmGet, MutData> {
        if key <= self.inner.blocks.count() {
            Ok(Entry {
                index: key,
                data: unsafe {
                    let Data { imm_data, mut_data } = self.inner.blocks.get(key);
                    Data {
                        imm_data: transmute::<&ImmData, &'imm ImmData>(imm_data),
                        mut_data: mut_data.clone(),
                    }
                },
            })
        } else {
            Err(KeyError)
        }
    }

    #[inline(always)]
    fn brw(&self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &MutData> {
        if key <= self.inner.blocks.count() {
            Ok(Entry {
                index: key,
                data: unsafe {
                    let Data { imm_data, mut_data } = self.inner.blocks.get(key);
                    Data { imm_data, mut_data }
                },
            })
        } else {
            Err(KeyError)
        }
    }

    #[inline(always)]
    fn brw_mut(&mut self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &mut MutData> {
        if key <= self.inner.blocks.count() {
            Ok(Entry {
                index: key,
                data: unsafe {
                    let Data { imm_data, mut_data } = self.inner.blocks.get_mut(key);
                    Data { imm_data, mut_data }
                },
            })
        } else {
            Err(KeyError)
        }
    }

    #[inline(always)]
    fn conv_get(get: Self::ImmGet) -> ImmData {
        get.clone()
    }

    #[inline(always)]
    fn scan_brw<'brw>(&'brw self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + 'brw {
        self.scan_get()
    }

    #[inline(always)]
    fn scan_get(&self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> {
        0..self.inner.blocks.count()
    }
    

    #[inline(always)]
    fn count(&self) -> usize {
        self.inner.blocks.count()
    }
    
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> PrimaryWindowApp<'imm, ImmData, MutData>
    for Window<'imm, AssocBlocks<ImmData, MutData, BLOCK_SIZE>>
where
    MutData: Clone,
    ImmData: Clone,
{
    #[inline(always)]
    fn append(&mut self, val: Data<ImmData, MutData>) -> <Self::Col as Keyable>::Key {
        let new_ind = self.inner.blocks.count();
        self.inner.blocks.append(val);
        new_ind
    }

    #[inline(always)]
    unsafe fn unppend(&mut self) {
        self.inner.blocks.unppend();
    }
}
