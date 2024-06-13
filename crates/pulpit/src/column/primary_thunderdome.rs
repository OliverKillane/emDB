use super::*;
use thunderdome::{Arena as ThunderArena, Index as ThunderIndex};

/// A Primary [`Column`] implemented using the [`thunderdome`]'s [`ThunderArena`].
/// - Conforms to the interface (using 8 byte [`UnsafeIndex`] indices) despite being
///   backed by [`u32`] indexed [`ThunderIndex`]s.
pub struct PrimaryThunderDome<ImmData, MutData> {
    arena: ThunderArena<Data<ImmData, MutData>>,
}

impl<ImmData, MutData> Column for PrimaryThunderDome<ImmData, MutData> {
    type WindowKind<'imm> = Window<'imm, PrimaryThunderDome<ImmData, MutData>>
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

impl<ImmData, MutData> Keyable for PrimaryThunderDome<ImmData, MutData> {
    type Key = ThunderIndex;
}

impl<'imm, ImmData, MutData> PrimaryWindow<'imm, ImmData, MutData>
    for Window<'imm, PrimaryThunderDome<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmGet = ImmData;
    type Col = PrimaryThunderDome<ImmData, MutData>;

    #[inline(always)]
    fn get(&self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmGet, &MutData> {
        let Entry {
            data: Data { imm_data, mut_data },
            index: _,
        } = self.brw(key)?;
        Ok(Entry {
            index: key.slot() as usize,
            data: Data {
                imm_data: imm_data.clone(),
                mut_data,
            },
        })
    }

    #[inline(always)]
    fn brw(&self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &MutData> {
        match self.inner.arena.get(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                index: key.slot() as usize,
                data: Data { imm_data, mut_data },
            }),
            None => Err(KeyError),
        }
    }

    #[inline(always)]
    fn brw_mut(&mut self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &mut MutData> {
        match self.inner.arena.get_mut(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                index: key.slot() as usize,
                data: Data { imm_data, mut_data },
            }),
            None => Err(KeyError),
        }
    }

    fn conv_get(get: Self::ImmGet) -> ImmData {
        get
    }

    #[inline(always)]
    fn scan_brw<'brw>(&'brw self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + 'brw {
        self.inner.arena.iter().map(|(i, _)| i)
    }

    #[inline(always)]
    fn scan_get(&self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + 'static {
        self.scan_brw().collect::<Vec<_>>().into_iter()
    }

    fn count(&self) -> usize {
        self.inner.arena.len()
    }
}

impl<'imm, ImmData, MutData> PrimaryWindowPull<'imm, ImmData, MutData>
    for Window<'imm, PrimaryThunderDome<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmPull = ImmData;

    #[inline(always)]
    fn insert(
        &mut self,
        val: Data<ImmData, MutData>,
    ) -> (<Self::Col as Keyable>::Key, InsertAction) {
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

    #[inline(always)]
    fn pull(&mut self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmPull, MutData> {
        match self.inner.arena.remove(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                index: key.slot() as usize,
                data: Data { imm_data, mut_data },
            }),
            None => Err(KeyError),
        }
    }

    fn conv_pull(pull: Self::ImmPull) -> ImmData {
        pull
    }
}
