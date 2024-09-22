use utils::Blocks;

use super::*;

/// Like a PrimaryRetain, but as an associated index.
pub struct AssocPullBlocks<ImmData, MutData, const BLOCK_SIZE: usize> {
    data: Vec<Data<PtrGen<ImmData>, MutData>>,
    blocks: Blocks<ImmData, BLOCK_SIZE>,
    _holder: (),
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Column
    for AssocPullBlocks<ImmData, MutData, BLOCK_SIZE>
{
    type WindowKind<'imm>
        = Window<'imm, Self>
    where
        Self: 'imm;

    fn new(size_hint: usize) -> Self {
        AssocPullBlocks {
            data: Vec::with_capacity(size_hint),
            blocks: Blocks::new(size_hint),
            _holder: (),
        }
    }

    fn window(&mut self) -> Self::WindowKind<'_> {
        Window { inner: self }
    }
}

impl<'imm, MutData, ImmData, const BLOCK_SIZE: usize> AssocWindow<'imm, ImmData, MutData>
    for Window<'imm, AssocPullBlocks<ImmData, MutData, BLOCK_SIZE>>
where
    ImmData: Clone,
{
    type ImmGet = &'imm ImmData;

    #[inline(always)]
    unsafe fn assoc_get(&self, ind: UnsafeIndex) -> Data<Self::ImmGet, &MutData> {
        let Data {
            imm_data: imm_ptr,
            mut_data,
        } = self.inner.data.get_unchecked(ind);
        Data {
            imm_data: &*(imm_ptr.0).cast::<ImmData>(),
            mut_data,
        }
    }

    #[inline(always)]
    unsafe fn assoc_brw(&self, ind: UnsafeIndex) -> Data<&ImmData, &MutData> {
        let Data {
            imm_data: imm_ptr,
            mut_data,
        } = self.inner.data.get_unchecked(ind);
        Data {
            imm_data: &*(imm_ptr.0).cast::<ImmData>(),
            mut_data,
        }
    }

    #[inline(always)]
    unsafe fn assoc_brw_mut(&mut self, ind: UnsafeIndex) -> Data<&ImmData, &mut MutData> {
        let Data {
            imm_data: imm_ptr,
            mut_data,
        } = self.inner.data.get_unchecked_mut(ind);
        Data {
            imm_data: &*(imm_ptr.0).cast::<ImmData>(),
            mut_data,
        }
    }

    #[inline(always)]
    fn assoc_append(&mut self, Data { imm_data, mut_data }: Data<ImmData, MutData>) {
        let ptr = self.inner.blocks.append(imm_data);
        self.inner.data.push(Data {
            imm_data: PtrGen(ptr),
            mut_data,
        });
    }

    #[inline(always)]
    unsafe fn assoc_unppend(&mut self) {
        self.inner.blocks.unppend();
        self.inner.data.pop();
    }

    fn conv_get(get: Self::ImmGet) -> ImmData {
        get.clone()
    }
}
