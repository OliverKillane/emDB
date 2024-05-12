use std::{marker::PhantomData, mem::transmute};

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

pub struct GenKey<Store, GenCounter: Copy + Eq> {
    index: ColInd,
    generation: GenCounter,
    phantom: PhantomData<Store>,
}

impl<Store, GenCounter: Copy + Eq> PartialEq for GenKey<Store, GenCounter> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}
impl<Store, GenCounter: Copy + Eq> Eq for GenKey<Store, GenCounter> {}
impl<Store, GenCounter: Copy + Eq> Clone for GenKey<Store, GenCounter> {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            generation: self.generation.clone(),
            phantom: PhantomData,
        }
    }
}
impl<Store, GenCounter: Copy + Eq> Copy for GenKey<Store, GenCounter> {}

mod assoc_blocks;
mod assoc_vec;
mod primary_no_pull;
mod primary_pull;
mod primary_retain;

mod utils {
    use std::mem::MaybeUninit;

    pub struct Blocks<Value, const BLOCK_SIZE: usize> {
        count: usize,
        data: Vec<Box<[MaybeUninit<Value>; BLOCK_SIZE]>>,
    }

    impl<Value, const BLOCK_SIZE: usize> Drop for Blocks<Value, BLOCK_SIZE> {
        fn drop(&mut self) {
            for alive in 0..self.count {
                let (block, seq) = quotrem::<BLOCK_SIZE>(alive);
                unsafe {
                    self.data.get_unchecked_mut(block)[seq].assume_init_drop();
                }
            }
        }
    }

    impl<Value, const BLOCK_SIZE: usize> Blocks<Value, BLOCK_SIZE> {
        pub fn new(size_hint: usize) -> Self {
            Blocks {
                count: 0,
                data: Vec::with_capacity(size_hint / BLOCK_SIZE + 1),
            }
        }

        pub fn append(&mut self, val: Value) -> *mut Value {
            let (block, seq) = quotrem::<BLOCK_SIZE>(self.count);
            let data_ptr;
            unsafe {
                if seq == 0 {
                    self.data
                        .push(Box::new(MaybeUninit::uninit().assume_init()));
                }
                data_ptr = self.data.get_unchecked_mut(block)[seq].as_mut_ptr();
                data_ptr.write(val);
            }
            self.count += 1;
            data_ptr
        }

        pub unsafe fn get(&self, ind: usize) -> &Value {
            let (block, seq) = quotrem::<BLOCK_SIZE>(ind);
            self.data.get_unchecked(block)[seq].assume_init_ref()
        }

        pub unsafe fn get_mut(&mut self, ind: usize) -> &mut Value {
            let (block, seq) = quotrem::<BLOCK_SIZE>(ind);
            self.data.get_unchecked_mut(block)[seq].assume_init_mut()
        }
    }

    pub fn quotrem<const DIV: usize>(val: usize) -> (usize, usize) {
        (val / DIV, val % DIV)
    }
}
