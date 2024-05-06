use std::cell::UnsafeCell;

pub enum ColumnError {
    AllocationFailure,
}

type ColInd = usize;

trait ColumnWindow<'imm, ImmStore, MutStore> {
    type GetVal<'brw>;
    type BrwVal<'brw>;
    
    type Col;

    fn new_col() -> Self::Col;
    fn new_view(col: &'imm mut Self::Col) -> Self;

    unsafe fn get<'brw>(&self, ind: ColInd) -> (Self::GetVal<'imm>, MutStore);
    unsafe fn brw<'brw>(&self, ind: ColInd) -> (Self::BrwVal<'brw>, &'brw MutStore);
    unsafe fn brw_mut<'brw>(&'brw mut self, ind: ColInd) -> &'brw mut MutStore;
    fn put_new(&mut self, x: (MutStore, ImmStore)) -> Result<(), ColumnError>;
}

trait ColumnWindowPull<'imm, ImmStore, MutStore>: ColumnWindow<'imm, ImmStore, MutStore> {
    unsafe fn pull(&mut self, ind: ColInd) -> (ImmStore, MutStore);
    unsafe fn place(&mut self, ind: ColInd, x: (MutStore, ImmStore));
}
