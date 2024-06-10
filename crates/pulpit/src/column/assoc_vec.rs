use assume::assume;

// TODO: Group together accesses into a hot block, then transfer these to the vector as it grows.
use super::*;

/// An associated, append only [`Column`] storing data in a large vector for faster
/// lookup than [`super::AssocBlocks`], but at the expense of needing copies for [`AssocWindow::assoc_get`].
pub struct AssocVec<ImmData, MutData> {
    data: Vec<Option<Data<ImmData, MutData>>>,
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

    #[inline(always)]
    unsafe fn assoc_get(&self, ind: UnsafeIndex) -> Data<Self::ImmGet, MutData> {
        let Data { imm_data, mut_data } = self.assoc_brw(ind);
        Data {
            imm_data: imm_data.clone(),
            mut_data: mut_data.clone(),
        }
    }

    #[inline(always)]
    unsafe fn assoc_brw(&self, ind: UnsafeIndex) -> Data<&ImmData, &MutData> {
        if let Some(Data { imm_data, mut_data }) = self.inner.data.get_unchecked(ind) {
            Data { imm_data, mut_data }
        } else {
            assume!(unsafe: @unreachable)
        }
    }

    #[inline(always)]
    unsafe fn assoc_brw_mut(&mut self, ind: UnsafeIndex) -> Data<&ImmData, &mut MutData> {
        if let Some(Data { imm_data, mut_data }) = self.inner.data.get_unchecked_mut(ind) {
            Data { imm_data, mut_data }
        } else {
            assume!(unsafe: @unreachable)
        }
    }

    #[inline(always)]
    fn assoc_append(&mut self, val: Data<ImmData, MutData>) {
        self.inner.data.push(Some(val))
    }

    #[inline(always)]
    fn conv_get(get: Self::ImmGet) -> ImmData {
        get
    }

    #[inline(always)]
    unsafe fn assoc_unppend(&mut self) {
        self.inner.data.pop();
    }
}

impl<'imm, ImmData, MutData> AssocWindowPull<'imm, ImmData, MutData>
    for Window<'imm, AssocVec<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmPull = ImmData;

    #[inline(always)]
    unsafe fn assoc_pull(&mut self, ind: UnsafeIndex) -> Data<Self::ImmPull, MutData> {
        let val = self.inner.data.get_unchecked_mut(ind);
        val.take().unwrap()
    }

    #[inline(always)]
    unsafe fn assoc_place(&mut self, ind: UnsafeIndex, val: Data<ImmData, MutData>) {
        *self.inner.data.get_unchecked_mut(ind) = Some(val);
    }

    #[inline(always)]
    fn conv_pull(pull: Self::ImmPull) -> ImmData {
        pull
    }
}
