use std::mem::{transmute, MaybeUninit};

use super::{
    utils::{push_new_block, quotrem},
    Column, ColumnWindow,
};

pub struct ColBlok<ImmData, MutData, const BLOCK_SIZE: usize> {
    count: usize,
    #[allow(clippy::type_complexity)]
    data: Vec<Box<[MaybeUninit<(ImmData, MutData)>; BLOCK_SIZE]>>,
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Drop for ColBlok<ImmData, MutData, BLOCK_SIZE> {
    fn drop(&mut self) {
        for alive in 0..self.count {
            let (block, seq) = quotrem::<BLOCK_SIZE>(alive);
            unsafe {
                self.data.get_unchecked_mut(block)[seq].assume_init_drop();
            }
        }
    }
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Column<ImmData, MutData>
    for ColBlok<ImmData, MutData, BLOCK_SIZE>
where
    MutData: Clone,
{
    type Window<'imm> = ColBlokWindow<'imm, ImmData, MutData, BLOCK_SIZE> where ImmData: 'imm, MutData: 'imm;
    type InitData = usize;

    fn new(size_hint: Self::InitData) -> Self {
        ColBlok {
            count: 0,
            data: Vec::with_capacity(size_hint / BLOCK_SIZE + 1),
        }
    }
}

pub struct ColBlokWindow<'imm, ImmData, MutData, const BLOCK_SIZE: usize> {
    col: &'imm mut ColBlok<ImmData, MutData, BLOCK_SIZE>,
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> ColumnWindow<'imm, ImmData, MutData>
    for ColBlokWindow<'imm, ImmData, MutData, BLOCK_SIZE>
where
    MutData: Clone,
{
    type GetVal = &'imm ImmData;
    type Col = ColBlok<ImmData, MutData, BLOCK_SIZE>;

    fn new_view(col: &'imm mut Self::Col) -> Self {
        Self { col }
    }

    unsafe fn get<'brw>(&'brw self, ind: super::ColInd) -> (Self::GetVal, MutData) {
        unsafe {
            let (imm_data, mut_data) = self.brw(ind);
            (
                transmute::<&'brw ImmData, &'imm ImmData>(imm_data),
                mut_data.clone(),
            )
        }
    }

    unsafe fn brw(&self, ind: super::ColInd) -> (&ImmData, &MutData) {
        let (block, seq) = quotrem::<BLOCK_SIZE>(ind);
        unsafe {
            let (imm_data, mut_data) = self.col.data.get_unchecked(block)[seq].assume_init_ref();
            (imm_data, mut_data)
        }
    }

    unsafe fn brw_mut(&mut self, ind: super::ColInd) -> &mut MutData {
        let (block, seq) = quotrem::<BLOCK_SIZE>(ind);
        unsafe {
            let (_, mut_data) = self.col.data.get_unchecked_mut(block)[seq].assume_init_mut();
            mut_data
        }
    }

    fn put_new(&mut self, x: (ImmData, MutData)) {
        unsafe {
            push_new_block::<(ImmData, MutData), BLOCK_SIZE>(
                &mut self.col.count,
                x,
                &mut self.col.data,
            )
        };
    }
}

#[cfg(kani)]
mod kani_verif {
    use super::*;

    #[kani::proof]
    #[kani::unwind(20)]
    fn check_get() {
        let mut col: ColBlok<(i32, String), usize, 1024> = ColBlok::new(0);
        let mut win =
            <ColBlok<(i32, String), usize, 1024> as Column<(i32, String), usize>>::Window::new_view(
                &mut col,
            );

        let mut strings = Vec::new();

        for i in 0..10 {
            win.put_new(((3, "hello_world".to_owned()), 123));
            let ((n, s), u) = unsafe { win.get(i) };

            let u_ref = unsafe { win.brw_mut(i) };

            *u_ref += 100;

            strings.push(s);
        }

        // we still have the strings
        for i in 0..10 {
            assert_eq!(strings[i], "hello_world");
        }
    }
}
