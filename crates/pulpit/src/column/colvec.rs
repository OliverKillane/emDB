//! # Vector based Column
//!

use super::{Column, ColumnWindow};

pub struct ColVec<ImmData, MutData> {
    data: Vec<(ImmData, MutData)>,
}

impl<ImmData, MutData> Column<ImmData, MutData> for ColVec<ImmData, MutData>
where
    ImmData: Clone,
    MutData: Clone,
{
    type Window<'imm> = ColVecWindow<'imm, ImmData, MutData> where ImmData: 'imm, MutData: 'imm;
    type InitData = usize;

    fn new(size_hint: Self::InitData) -> Self {
        ColVec {
            data: Vec::with_capacity(size_hint),
        }
    }
}

pub struct ColVecWindow<'imm, ImmData, MutData>
where
    ImmData: Clone,
    MutData: Clone,
{
    data: &'imm mut Vec<(ImmData, MutData)>,
}

impl<'imm, ImmData, MutData> ColumnWindow<'imm, ImmData, MutData>
    for ColVecWindow<'imm, ImmData, MutData>
where
    ImmData: Clone,
    MutData: Clone,
{
    type GetVal<'brw> = ImmData;
    type Col = ColVec<ImmData, MutData>;

    fn new_view(col: &'imm mut Self::Col) -> Self {
        ColVecWindow {
            data: &mut col.data,
        }
    }

    unsafe fn get<'brw>(&'brw self, ind: super::ColInd) -> (Self::GetVal<'imm>, MutData) {
        let (imm, muta) = &self.data[ind];
        (imm.clone(), muta.clone())
    }

    unsafe fn brw(&self, ind: super::ColInd) -> (&ImmData, &MutData) {
        let (imm, muta) = &self.data[ind];
        (imm, muta)
    }

    unsafe fn brw_mut(&mut self, ind: super::ColInd) -> &mut MutData {
        &mut self.data[ind].1
    }

    fn put_new(&mut self, x: (ImmData, MutData)) {
        self.data.push(x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colvec() {
        let mut col = ColVec::<i32, i32>::new(10);
        let mut window = ColVecWindow::new_view(&mut col);
        window.put_new((1, 2));
        let (imm, muta) = unsafe { window.get(0) };
        assert_eq!(imm, 1);
        assert_eq!(muta, 2);
    }
}
