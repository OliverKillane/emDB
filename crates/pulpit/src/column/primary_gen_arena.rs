use super::*;
use typed_generational_arena::{Arena as GenArena, Index as GenIndex};

/// A Primary [`Column`] implemented using the [`typed_generational_arena`]'s [`GenArena`].
/// - No immutability optimisations.
pub struct PrimaryGenerationalArena<ImmData, MutData> {
    arena: GenArena<Data<ImmData, MutData>>,
}

impl<ImmData, MutData> Column for PrimaryGenerationalArena<ImmData, MutData> {
    type WindowKind<'imm> = Window<'imm, PrimaryGenerationalArena<ImmData, MutData>>
    where
        Self: 'imm;

    fn new(size_hint: usize) -> Self {
        Self {
            arena: GenArena::with_capacity(size_hint),
        }
    }

    fn window(&mut self) -> Self::WindowKind<'_> {
        Window { inner: self }
    }
}

impl<'imm, ImmData, MutData> PrimaryWindow<'imm, ImmData, MutData>
    for Window<'imm, PrimaryGenerationalArena<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmGet = ImmData;
    type Key = GenIndex<Data<ImmData, MutData>, usize, usize>;

    fn get(&self, key: Self::Key) -> Access<Self::ImmGet, MutData> {
        let Entry {
            data: Data { imm_data, mut_data },
            index,
        } = self.brw(key)?;
        Ok(Entry {
            index,
            data: Data {
                imm_data: imm_data.clone(),
                mut_data: mut_data.clone(),
            },
        })
    }

    fn brw(&self, key: Self::Key) -> Access<&ImmData, &MutData> {
        match self.inner.arena.get(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                data: Data { imm_data, mut_data },
                index: key.to_idx(),
            }),
            None => Err(KeyError),
        }
    }

    fn brw_mut(&mut self, key: Self::Key) -> Access<&ImmData, &mut MutData> {
        match self.inner.arena.get_mut(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                data: Data { imm_data, mut_data },
                index: key.to_idx(),
            }),
            None => Err(KeyError),
        }
    }
}

impl<'imm, ImmData, MutData> PrimaryWindowPull<'imm, ImmData, MutData>
    for Window<'imm, PrimaryGenerationalArena<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmPull = ImmData;

    fn insert(&mut self, val: Data<ImmData, MutData>) -> (Self::Key, InsertAction) {
        let curr_max = self.inner.arena.len();
        let key = self.inner.arena.insert(val);
        (
            key,
            if key.to_idx() == curr_max {
                InsertAction::Append
            } else {
                InsertAction::Place(key.to_idx())
            },
        )
    }

    fn pull(&mut self, key: Self::Key) -> Access<Self::ImmPull, MutData> {
        match self.inner.arena.remove(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                data: Data { imm_data, mut_data },
                index: key.to_idx(),
            }),
            None => Err(KeyError),
        }
    }
}
