use std::mem::transmute;

/// A single window type holding a mutable references through which windows for
/// columns and primary indexes can be generated.
pub struct Window<'imm, Data> {
    inner: &'imm mut Data,
}

pub trait Store {
    type WindowKind<'imm>
    where
        Self: 'imm;
    fn new(size_hint: usize) -> Self;
    fn window<'imm>(&'imm mut self) -> Self::WindowKind<'imm>;
}

pub type ColInd = usize;

pub struct Data<ImmData, MutData> {
    pub imm_data: ImmData,
    pub mut_data: MutData,
}

pub struct Entry<ImmData, MutData> {
    pub index: ColInd,
    pub data: Data<ImmData, MutData>,
}

pub type Access<Imm, Mut> = Result<Entry<Imm, Mut>, KeyError>;
pub enum InsertAction {
    Place(ColInd),
    Append,
}
pub struct KeyError;

pub trait PrimaryWindow<'imm, ImmData, MutData> {
    type ImmGet;
    type Key: Copy + Eq;

    fn get(&self, key: Self::Key) -> Access<Self::ImmGet, MutData>;
    fn brw(&self, key: Self::Key) -> Access<&ImmData, &MutData>;
    fn brw_mut(&mut self, key: Self::Key) -> Access<&ImmData, &mut MutData>;
}

pub trait PrimaryWindowApp<'imm, ImmData, MutData>: PrimaryWindow<'imm, ImmData, MutData> {
    fn append(&mut self, val: Data<ImmData, MutData>) -> Self::Key;
}

pub trait PrimaryWindowPull<'imm, ImmData, MutData>: PrimaryWindow<'imm, ImmData, MutData> {
    type ImmPull;
    fn insert(&mut self, val: Data<ImmData, MutData>) -> (Self::Key, InsertAction);
    fn pull(&mut self, key: Self::Key) -> Access<Self::ImmPull, MutData>;
}

pub trait AssocWindow<'imm, ImmData, MutData> {
    type ImmGet;
    unsafe fn get(&self, ind: ColInd) -> Data<Self::ImmGet, MutData>;
    unsafe fn brw(&self, ind: ColInd) -> Data<&ImmData, &MutData>;
    unsafe fn brw_mut(&mut self, ind: ColInd) -> Data<&ImmData, &mut MutData>;
    fn append(&mut self, val: Data<ImmData, MutData>);
}

pub trait AssocWindowPull<'imm, ImmData, MutData>: AssocWindow<'imm, ImmData, MutData> {
    type ImmPull;
    unsafe fn pull(&mut self, ind: ColInd) -> Data<Self::ImmPull, MutData>;
    unsafe fn place(&mut self, ind: ColInd, val: Data<ImmData, MutData>);
}

mod assoc_blocks;
mod assoc_map;
mod assoc_vec;
mod primary_no_pull;
mod primary_pull;
mod primary_retain;
