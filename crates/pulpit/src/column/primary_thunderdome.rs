use super::*;
use thunderdome::{Arena as ThunderArena, Index as ThunderIndex};

/// A Primary [`Column`] implemented using the [`thunderdome`]'s [`ThunderArena`].
/// - Conforms to the interface (using 8 byte [`UnsafeIndex`] indices) despite being
///   backed by [`u32`] indexed [`ThunderIndex`]s.
pub struct ThunderDome<ImmData, MutData> {
    arena: ThunderArena<Data<ImmData, MutData>>,
}

impl<ImmData, MutData> Column for ThunderDome<ImmData, MutData> {
    type WindowKind<'imm> = Window<'imm, ThunderDome<ImmData, MutData>>
    where
        Self: 'imm;

    fn new(size_hint: usize) -> Self {
        Self {
            arena: ThunderArena::with_capacity(size_hint),
        }
    }

    fn window(&mut self) -> Self::WindowKind<'_> {
        Window { inner: self }
    }
}

impl<'imm, ImmData, MutData> PrimaryWindow<'imm, ImmData, MutData>
    for Window<'imm, ThunderDome<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmGet = ImmData;
    type Key = ThunderIndex;

    fn get(&self, key: Self::Key) -> Access<Self::ImmGet, MutData> {
        let Entry {
            data: Data { imm_data, mut_data },
            index,
        } = self.brw(key)?;
        Ok(Entry {
            index: key.slot() as usize,
            data: Data {
                imm_data: imm_data.clone(),
                mut_data: mut_data.clone(),
            },
        })
    }

    fn brw(&self, key: Self::Key) -> Access<&ImmData, &MutData> {
        match self.inner.arena.get(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                index: key.slot() as usize,
                data: Data { imm_data, mut_data },
            }),
            None => Err(KeyError),
        }
    }

    fn brw_mut(&mut self, key: Self::Key) -> Access<&ImmData, &mut MutData> {
        match self.inner.arena.get_mut(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                index: key.slot() as usize,
                data: Data { imm_data, mut_data },
            }),
            None => Err(KeyError),
        }
    }
}

impl<'imm, ImmData, MutData> PrimaryWindowPull<'imm, ImmData, MutData>
    for Window<'imm, ThunderDome<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmPull = ImmData;

    fn insert(&mut self, val: Data<ImmData, MutData>) -> (Self::Key, InsertAction) {
        let curr_max = self.inner.arena.len();
        let key = self.inner.arena.insert(val);
        let index = key.slot() as usize;
        (
            key,
            if index == curr_max {
                InsertAction::Append
            } else {
                InsertAction::Place(index)
            },
        )
    }

    fn pull(&mut self, key: Self::Key) -> Access<Self::ImmPull, MutData> {
        match self.inner.arena.remove(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                index: key.slot() as usize,
                data: Data { imm_data, mut_data },
            }),
            None => Err(KeyError),
        }
    }
}
