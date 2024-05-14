// TODO: Group together accesses into a hot block, then transfer these to the vector as it grows.
use super::*;

/// An associated, append only [`Column`] storing data in a large vector for faster
/// lookup than [`ColBlok`], but at the expense of needing copies for [`AssocWindow::get`].
pub struct AssocVec<ImmData, MutData> {
    data: Vec<Data<ImmData, MutData>>,
}

impl<ImmData, MutData> Column for AssocVec<ImmData, MutData> {
    type WindowKind<'imm> = Window<'imm, AssocVec<ImmData, MutData>>
    where
        Self: 'imm;

    fn new(size_hint: usize) -> Self {
        AssocVec {
            data: Vec::with_capacity(size_hint),
        }
    }

    fn window(&mut self) -> Self::WindowKind<'_> {
        Window { inner: self }
    }
}

impl<'imm, ImmData, MutData> AssocWindow<'imm, ImmData, MutData>
    for Window<'imm, AssocVec<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmGet = ImmData;

    unsafe fn get(&self, ind: UnsafeIndex) -> Data<Self::ImmGet, MutData> {
        let Data { imm_data, mut_data } = self.brw(ind);
        Data {
            imm_data: imm_data.clone(),
            mut_data: mut_data.clone(),
        }
    }

    unsafe fn brw(&self, ind: UnsafeIndex) -> Data<&ImmData, &MutData> {
        let Data { imm_data, mut_data } = self.inner.data.get_unchecked(ind);
        Data { imm_data, mut_data }
    }

    unsafe fn brw_mut(&mut self, ind: UnsafeIndex) -> Data<&ImmData, &mut MutData> {
        let Data { imm_data, mut_data } = self.inner.data.get_unchecked_mut(ind);
        Data { imm_data, mut_data }
    }

    fn append(&mut self, val: Data<ImmData, MutData>) {
        self.inner.data.push(val)
    }

    fn conv_get(get: Self::ImmGet) -> ImmData {
        get
    }
}
