use super::*;
use thunderdome::{Arena as ThunderArena, Index as ThunderIndex};

struct TransData<MutData> {
    visible: bool,
    mut_data: MutData,
}

/// A modification on [`PrimaryThunderDome`] to allow for transactions by nincluding a 'hide field'
pub struct PrimaryThunderDomeTrans<ImmData, MutData> {
    arena: ThunderArena<Data<ImmData, TransData<MutData>>>,
    visible_size: usize,
}

impl<ImmData, MutData> Column for PrimaryThunderDomeTrans<ImmData, MutData> {
    type WindowKind<'imm> = Window<'imm, PrimaryThunderDomeTrans<ImmData, MutData>>
    where
        Self: 'imm;

    #[inline(always)]
    fn new(size_hint: usize) -> Self {
        Self {
            arena: ThunderArena::with_capacity(size_hint),
            visible_size: 0,
        }
    }

    #[inline(always)]
    fn window(&mut self) -> Self::WindowKind<'_> {
        Window { inner: self }
    }
}

impl<ImmData, MutData> Keyable for PrimaryThunderDomeTrans<ImmData, MutData> {
    type Key = ThunderIndex;
}

impl<'imm, ImmData, MutData> PrimaryWindow<'imm, ImmData, MutData>
    for Window<'imm, PrimaryThunderDomeTrans<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmGet = ImmData;
    type Col = PrimaryThunderDomeTrans<ImmData, MutData>;

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
                mut_data: mut_data,
            },
        })
    }

    #[inline(always)]
    fn brw(&self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &MutData> {
        match self.inner.arena.get(key) {
            Some(Data {
                imm_data,
                mut_data:
                    TransData {
                        visible: true,
                        mut_data,
                    },
            }) => Ok(Entry {
                index: key.slot() as usize,
                data: Data { imm_data, mut_data },
            }),
            _ => Err(KeyError),
        }
    }

    #[inline(always)]
    fn brw_mut(&mut self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &mut MutData> {
        match self.inner.arena.get_mut(key) {
            Some(Data {
                imm_data,
                mut_data:
                    TransData {
                        visible: true,
                        mut_data,
                    },
            }) => Ok(Entry {
                index: key.slot() as usize,
                data: Data { imm_data, mut_data },
            }),
            _ => Err(KeyError),
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
    for Window<'imm, PrimaryThunderDomeTrans<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmPull = ImmData;

    #[inline(always)]
    fn insert(
        &mut self,
        Data { imm_data, mut_data }: Data<ImmData, MutData>,
    ) -> (<Self::Col as Keyable>::Key, InsertAction) {
        let curr_max = self.inner.arena.len();
        let key = self.inner.arena.insert(Data {
            imm_data,
            mut_data: TransData {
                visible: true,
                mut_data,
            },
        });
        self.inner.visible_size += 1;
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
            Some(Data {
                imm_data,
                mut_data:
                    TransData {
                        visible: true,
                        mut_data,
                    },
            }) => {
                self.inner.visible_size -= 1;
                Ok(Entry {
                    index: key.slot() as usize,
                    data: Data { imm_data, mut_data },
                })
            }
            _ => Err(KeyError),
        }
    }

    fn conv_pull(pull: Self::ImmPull) -> ImmData {
        pull
    }
}

impl<'imm, ImmData, MutData> PrimaryWindowHide<'imm, ImmData, MutData>
    for Window<'imm, PrimaryThunderDomeTrans<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    #[inline(always)]
    fn hide(&mut self, key: <Self::Col as Keyable>::Key) -> Result<(), KeyError> {
        match self.inner.arena.get_mut(key) {
            Some(Data {
                imm_data: _,
                mut_data:
                    TransData {
                        visible,
                        mut_data: _,
                    },
            }) if *visible => {
                *visible = false;
                self.inner.visible_size -= 1;
                Ok(())
            }
            _ => Err(KeyError),
        }
    }

    #[inline(always)]
    fn reveal(&mut self, key: <Self::Col as Keyable>::Key) -> Result<(), KeyError> {
        match self.inner.arena.get_mut(key) {
            Some(Data {
                imm_data: _,
                mut_data:
                    TransData {
                        visible,
                        mut_data: _,
                    },
            }) if !*visible => {
                *visible = true;
                self.inner.visible_size += 1;
                Ok(())
            }
            _ => Err(KeyError),
        }
    }
}
