use super::*;

/// An associated, append only [`Column`] that stores mutable and immutable data together in
/// blocks, and provides stable references to the immutable part.
pub struct ColBlok<ImmData, MutData, const BLOCK_SIZE: usize> {
    blocks: utils::Blocks<Data<ImmData, MutData>, BLOCK_SIZE>,
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Column for ColBlok<ImmData, MutData, BLOCK_SIZE> {
    type WindowKind<'imm> = Window<'imm, ColBlok<ImmData, MutData, BLOCK_SIZE>>
    where
        Self: 'imm;

    fn new(size_hint: usize) -> Self {
        ColBlok {
            blocks: utils::Blocks::new(size_hint),
        }
    }

    fn window(&mut self) -> Self::WindowKind<'_> {
        Window { inner: self }
    }
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> AssocWindow<'imm, ImmData, MutData>
    for Window<'imm, ColBlok<ImmData, MutData, BLOCK_SIZE>>
where
    MutData: Clone,
{
    type ImmGet = &'imm ImmData;

    unsafe fn get(&self, ind: UnsafeIndex) -> Data<Self::ImmGet, MutData> {
        unsafe {
            let Data { imm_data, mut_data } = self.brw(ind);
            Data {
                imm_data: transmute::<&ImmData, &'imm ImmData>(imm_data),
                mut_data: mut_data.clone(),
            }
        }
    }

    unsafe fn brw(&self, ind: UnsafeIndex) -> Data<&ImmData, &MutData> {
        unsafe {
            let Data { imm_data, mut_data } = self.inner.blocks.get(ind);
            Data { imm_data, mut_data }
        }
    }

    unsafe fn brw_mut(&mut self, ind: UnsafeIndex) -> Data<&ImmData, &mut MutData> {
        unsafe {
            let Data { imm_data, mut_data } = self.inner.blocks.get_mut(ind);
            Data { imm_data, mut_data }
        }
    }

    fn append(&mut self, val: Data<ImmData, MutData>) {
        self.inner.blocks.append(val);
    }
}
