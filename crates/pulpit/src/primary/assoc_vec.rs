use super::*;

pub struct VecAssoc<ImmData, MutData> {
    data: Vec<Data<ImmData, MutData>>,
}

impl<ImmData, MutData> Store for VecAssoc<ImmData, MutData> {
    type WindowKind<'imm> = Window<'imm, VecAssoc<ImmData, MutData>>
    where
        Self: 'imm;

    fn new(size_hint: usize) -> Self {
        VecAssoc {
            data: Vec::with_capacity(size_hint),
        }
    }

    fn window<'imm>(&'imm mut self) -> Self::WindowKind<'imm> {
        Window { inner: self }
    }
}

impl<'imm, ImmData, MutData> AssocWindow<'imm, ImmData, MutData>
    for Window<'imm, VecAssoc<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmGet = ImmData;

    unsafe fn get(&self, ind: ColInd) -> Data<Self::ImmGet, MutData> {
        let Data { imm_data, mut_data } = self.brw(ind);
        Data {
            imm_data: imm_data.clone(),
            mut_data: mut_data.clone(),
        }
    }

    unsafe fn brw(&self, ind: ColInd) -> Data<&ImmData, &MutData> {
        let Data { imm_data, mut_data } = self.inner.data.get_unchecked(ind);
        Data { imm_data, mut_data }
    }

    unsafe fn brw_mut(&mut self, ind: ColInd) -> Data<&ImmData, &mut MutData> {
        let Data { imm_data, mut_data } = self.inner.data.get_unchecked_mut(ind);
        Data { imm_data, mut_data }
    }

    fn append(&mut self, val: Data<ImmData, MutData>) {
        self.inner.data.push(val)
    }
}
