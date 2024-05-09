use super::{
    utils::{push_new_block, quotrem},
    Column, ColumnWindow, ColumnWindowPull,
};
use std::{
    mem::{transmute, MaybeUninit},
    ptr,
};

/// TODO: allow data to be stored in MutData state, and access to ptr from outside
/// (internal for the generational arena)

struct RetEntry<ImmData, MutData> {
    /// Pointer to the immutable data, if this entry is dead/pulled the pointer is [`ptr::null()`]
    imm_data_ptr: *const ImmData,
    mut_data: MaybeUninit<MutData>,
}

#[allow(rustdoc::private_intra_doc_links)]
/// An immutable retaining column.
///
/// Supports [`ColumnWindowPull`] while also providing stable immutable references
/// - Stores the immutable data separately in blocks ([`ColRet::imm_data`])
/// - Stores pointers to the immutable data, and the mutable data in blocks ([`ColRet::mut_and_ptrs`])
/// - When values are pulled, the immutable data is retained.
///
/// # Safety
/// - Entries are uninitialised until the block is expanded (for both [`ColRet::imm_data`] and [`ColRet::mut_and_ptrs`])
/// - In an initialised [`ColRet::mut_and_ptrs`] entry, the [`RetEntry::imm_data_ptr`] signifies if the enty is alive, or has been pulled (is is [`ptr::null()`] in)  
pub struct ColRet<ImmData, MutData, const BLOCK_SIZE: usize> {
    imm_data_count: usize,
    mut_and_ptrs_count: usize,
    imm_data: Vec<Box<[MaybeUninit<ImmData>; BLOCK_SIZE]>>,
    #[allow(clippy::type_complexity)]
    mut_and_ptrs: Vec<Box<[MaybeUninit<RetEntry<ImmData, MutData>>; BLOCK_SIZE]>>,
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Drop for ColRet<ImmData, MutData, BLOCK_SIZE> {
    fn drop(&mut self) {
        for alive in 0..self.imm_data_count {
            let (block, seq) = quotrem::<BLOCK_SIZE>(alive);
            unsafe {
                self.imm_data.get_unchecked_mut(block)[seq].assume_init_drop();
            }
        }
        for alive in 0..self.mut_and_ptrs_count {
            let (block, seq) = quotrem::<BLOCK_SIZE>(alive);

            unsafe {
                let entry = &mut self.mut_and_ptrs.get_unchecked_mut(block)[seq].assume_init_mut();
                if entry.imm_data_ptr != ptr::null() {
                    entry.mut_data.assume_init_drop();
                }
            }
        }
    }
}

impl<ImmData, MutData, const BLOCK_SIZE: usize> Column<ImmData, MutData>
    for ColRet<ImmData, MutData, BLOCK_SIZE>
where
    MutData: Clone,
{
    type Window<'imm> = ColRetWindow<'imm, ImmData, MutData, BLOCK_SIZE> where ImmData: 'imm, MutData: 'imm;
    type InitData = usize;

    fn new(size_hint: Self::InitData) -> Self {
        ColRet {
            imm_data_count: 0,
            mut_and_ptrs_count: 0,
            imm_data: Vec::with_capacity(size_hint / BLOCK_SIZE + 1),
            mut_and_ptrs: Vec::with_capacity(size_hint / BLOCK_SIZE + 1),
        }
    }
}

pub struct ColRetWindow<'imm, ImmData, MutData, const BLOCK_SIZE: usize> {
    col: &'imm mut ColRet<ImmData, MutData, BLOCK_SIZE>,
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> ColumnWindow<'imm, ImmData, MutData>
    for ColRetWindow<'imm, ImmData, MutData, BLOCK_SIZE>
where
    MutData: Clone,
{
    type GetVal = &'imm ImmData;

    type Col = ColRet<ImmData, MutData, BLOCK_SIZE>
    where
        ImmData: 'imm,
        MutData: 'imm;

    fn new_view(col: &'imm mut Self::Col) -> Self {
        Self { col }
    }

    unsafe fn get<'brw>(&'brw self, ind: super::ColInd) -> (Self::GetVal, MutData) {
        let (imm_data, mut_data) = self.brw(ind);
        (
            transmute::<&'brw ImmData, &'imm ImmData>(imm_data),
            mut_data.clone(),
        )
    }

    unsafe fn brw(&self, ind: super::ColInd) -> (&ImmData, &MutData) {
        let (block, seq) = quotrem::<BLOCK_SIZE>(ind);
        let entry = unsafe { &self.col.mut_and_ptrs.get_unchecked(block)[seq] };
        let RetEntry {
            imm_data_ptr,
            mut_data,
        } = entry.assume_init_ref();
        (
            &*(*imm_data_ptr).cast::<ImmData>(),
            mut_data.assume_init_ref(),
        )
    }

    unsafe fn brw_mut(&mut self, ind: super::ColInd) -> &mut MutData {
        let (block, seq) = quotrem::<BLOCK_SIZE>(ind);
        let RetEntry {
            imm_data_ptr,
            mut_data,
        } = self.col.mut_and_ptrs.get_unchecked_mut(block)[seq].assume_init_mut();
        mut_data.assume_init_mut()
    }

    fn put_new(&mut self, (imm_data, mut_data): (ImmData, MutData)) {
        unsafe {
            let imm_data_ptr =
                transmute::<*mut ImmData, *const ImmData>(push_new_block::<ImmData, BLOCK_SIZE>(
                    &mut self.col.imm_data_count,
                    imm_data,
                    &mut self.col.imm_data,
                ));
            push_new_block(
                &mut self.col.mut_and_ptrs_count,
                RetEntry {
                    imm_data_ptr,
                    mut_data: MaybeUninit::new(mut_data),
                },
                &mut self.col.mut_and_ptrs,
            );
        }
    }
}

impl<'imm, ImmData, MutData, const BLOCK_SIZE: usize> ColumnWindowPull<'imm, ImmData, MutData>
    for ColRetWindow<'imm, ImmData, MutData, BLOCK_SIZE>
where
    MutData: Clone,
{
    type PullVal = &'imm ImmData;

    unsafe fn pull(&mut self, ind: super::ColInd) -> (Self::PullVal, MutData) {
        let (block, seq) = quotrem::<BLOCK_SIZE>(ind);

        let entry_ref = &mut self.col.mut_and_ptrs.get_unchecked_mut(block)[seq];

        let imm_data_ptr = &mut entry_ref.assume_init_mut().imm_data_ptr;
        let imm_data_ptr_saved = *imm_data_ptr;
        *imm_data_ptr = ptr::null();

        let mut_data = entry_ref.assume_init_read().mut_data;
        (
            &*imm_data_ptr_saved.cast::<ImmData>(),
            mut_data.assume_init_read(),
        )
    }

    unsafe fn place(&mut self, ind: super::ColInd, (imm_data, mut_data): (ImmData, MutData)) {
        let (block, seq) = quotrem::<BLOCK_SIZE>(ind);
        let imm_data_ptr =
            transmute::<*mut ImmData, *const ImmData>(push_new_block::<ImmData, BLOCK_SIZE>(
                &mut self.col.imm_data_count,
                imm_data,
                &mut self.col.imm_data,
            ));
        self.col.mut_and_ptrs.get_unchecked_mut(block)[seq].write(RetEntry {
            imm_data_ptr,
            mut_data: MaybeUninit::new(mut_data),
        });
    }
}
