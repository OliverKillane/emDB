use super::*;
use typed_generational_arena::{Arena as GenArena, Index as GenIndex};

/// A Primary [`Column`] implemented using the [`typed_generational_arena`]'s [`GenArena`].
/// - No immutability optimisations.
pub struct PrimaryGenerationalArena<ImmData, MutData> {
    arena: GenArena<Data<ImmData, MutData>>,
}

impl<ImmData, MutData> Keyable for PrimaryGenerationalArena<ImmData, MutData> {
    type Key = GenIndex<Data<ImmData, MutData>, usize, usize>;
}

impl<ImmData, MutData> Column for PrimaryGenerationalArena<ImmData, MutData> {
    type WindowKind<'imm>
        = Window<'imm, PrimaryGenerationalArena<ImmData, MutData>>
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
    ImmData: Clone + 'static,
    MutData: Clone + 'static,
{
    type ImmGet = ImmData;
    type Col = PrimaryGenerationalArena<ImmData, MutData>;

    #[inline(always)]
    fn get(&self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmGet, &MutData> {
        let Entry {
            data: Data { imm_data, mut_data },
            index,
        } = self.brw(key)?;
        Ok(Entry {
            index,
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
                data: Data { imm_data, mut_data },
                index: key.to_idx(),
            }),
            None => Err(KeyError),
        }
    }

    #[inline(always)]
    fn brw_mut(&mut self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &mut MutData> {
        match self.inner.arena.get_mut(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                data: Data { imm_data, mut_data },
                index: key.to_idx(),
            }),
            None => Err(KeyError),
        }
    }

    #[inline(always)]
    fn conv_get(get: Self::ImmGet) -> ImmData {
        get
    }

    #[inline(always)]
    fn scan_brw(&self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + '_ {
        self.inner.arena.iter().map(|(key, _)| key)
    }

    fn scan_get(&self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + 'static {
        self.scan_brw().collect::<Vec<_>>().into_iter()
    }

    #[inline(always)]
    fn count(&self) -> usize {
        self.inner.arena.len()
    }
}

impl<'imm, ImmData, MutData> PrimaryWindowPull<'imm, ImmData, MutData>
    for Window<'imm, PrimaryGenerationalArena<ImmData, MutData>>
where
    ImmData: Clone + 'static,
    MutData: Clone + 'static,
{
    type ImmPull = ImmData;

    #[inline(always)]
    fn insert(
        &mut self,
        val: Data<ImmData, MutData>,
    ) -> (<Self::Col as Keyable>::Key, InsertAction) {
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

    #[inline(always)]
    fn pull(&mut self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmPull, MutData> {
        match self.inner.arena.remove(key) {
            Some(Data { imm_data, mut_data }) => Ok(Entry {
                data: Data { imm_data, mut_data },
                index: key.to_idx(),
            }),
            None => Err(KeyError),
        }
    }

    #[inline(always)]
    fn conv_pull(pull: Self::ImmPull) -> ImmData {
        pull
    }
}
