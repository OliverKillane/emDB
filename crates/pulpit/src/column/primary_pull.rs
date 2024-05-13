use std::marker::PhantomData;

use super::*;

enum GenEntry {
    Generation(usize),
    NextFree(Option<usize>),
}

type GenEntryEncoded = usize;

impl GenEntry {
    fn decode(val: &GenEntryEncoded) -> GenEntry {
        if val % 2 == 0 {
            GenEntry::Generation(val / 2)
        } else {
            let opt = val / 2;
            GenEntry::NextFree(if opt == 0 { None } else { Some(opt - 1) })
        }
    }

    fn encode(&self) -> GenEntryEncoded {
        match self {
            GenEntry::Generation(val) => {
                debug_assert!(*val <= usize::MAX / 2);
                val * 2
            }
            GenEntry::NextFree(opt) => opt
                .map(|val| {
                    debug_assert!(val < ((usize::MAX - 1) / 2));
                    (val + 1) * 2 + 1
                })
                .unwrap_or(1),
        }
    }
}

struct GenInfo {
    next_free: Option<usize>,
    gen_counter: usize,
    generations: Vec<GenEntryEncoded>,
}

impl GenInfo {
    fn lookup_key<Store>(&self, key: GenKey<Store, usize>) -> Result<UnsafeIndex, KeyError> {
        match self.generations.get(key.index).map(GenEntry::decode) {
            Some(GenEntry::Generation(g)) if key.generation == g => Ok(key.index),
            _ => Err(KeyError),
        }
    }

    fn pull_key<Store>(&mut self, key: GenKey<Store, usize>) -> Result<UnsafeIndex, KeyError> {
        if let Some(entry) = self.generations.get_mut(key.index) {
            match GenEntry::decode(&*entry) {
                GenEntry::Generation(g) if g == key.generation => {
                    *entry = GenEntry::NextFree(self.next_free).encode();
                    self.next_free = Some(key.index);
                    self.gen_counter += 1;
                    Ok(key.index)
                }
                _ => Err(KeyError),
            }
        } else {
            Err(KeyError)
        }
    }

    fn insert<Store>(&mut self) -> (GenKey<Store, usize>, InsertAction) {
        if let Some(k) = self.next_free {
            // TODO: could use unchecked here
            let entry = self.generations.get_mut(k).unwrap();
            match GenEntry::decode(&*entry) {
                GenEntry::NextFree(opt) => {
                    self.next_free = opt;
                    *entry = GenEntry::Generation(self.gen_counter).encode();
                    (
                        GenKey {
                            index: k,
                            generation: self.gen_counter,
                            phantom: PhantomData,
                        },
                        InsertAction::Place(k),
                    )
                }
                _ => unreachable!(),
            }
        } else {
            let index = self.generations.len();
            self.generations
                .push(GenEntry::Generation(self.gen_counter).encode());
            (
                GenKey {
                    index,
                    generation: self.gen_counter,
                    phantom: PhantomData,
                },
                InsertAction::Append,
            )
        }
    }
}

/// An adapter to convert an [`AssocWindowPull`] into a [`PrimaryWindowPull`] with generational indices.
pub struct PullWrap<Col> {
    col: Col,
    gen: GenInfo,
}

impl<Col: Column> Column for PullWrap<Col> {
    type WindowKind<'imm> =  PullWrapWindow<'imm, Col> where Self: 'imm;

    fn new(size_hint: usize) -> Self {
        PullWrap {
            col: Col::new(size_hint),
            gen: GenInfo {
                next_free: None,
                generations: Vec::with_capacity(size_hint),
                gen_counter: 0,
            },
        }
    }

    fn window(&mut self) -> Self::WindowKind<'_> {
        PullWrapWindow {
            col: self.col.window(),
            gen: &mut self.gen,
        }
    }
}

pub struct PullWrapWindow<'imm, Col: Column + 'imm> {
    col: Col::WindowKind<'imm>,
    gen: &'imm mut GenInfo,
}

impl<'imm, ImmData, MutData, Col> PrimaryWindow<'imm, ImmData, MutData>
    for PullWrapWindow<'imm, Col>
where
    Col: Column,
    Col::WindowKind<'imm>: AssocWindow<'imm, ImmData, MutData>,
{
    type ImmGet = <Col::WindowKind<'imm> as AssocWindow<'imm, ImmData, MutData>>::ImmGet;
    type Key = GenKey<PullWrap<Col>, usize>;

    fn get(&self, key: Self::Key) -> Access<Self::ImmGet, MutData> {
        let index = self.gen.lookup_key(key)?;
        Ok(Entry {
            index,
            data: unsafe { self.col.get(index) },
        })
    }

    fn brw(&self, key: Self::Key) -> Access<&ImmData, &MutData> {
        let index = self.gen.lookup_key(key)?;
        Ok(Entry {
            index,
            data: unsafe { self.col.brw(index) },
        })
    }

    fn brw_mut(&mut self, key: Self::Key) -> Access<&ImmData, &mut MutData> {
        let index = self.gen.lookup_key(key)?;
        Ok(Entry {
            index,
            data: unsafe { self.col.brw_mut(index) },
        })
    }
}

impl<'imm, ImmData, MutData, Col> PrimaryWindowPull<'imm, ImmData, MutData>
    for PullWrapWindow<'imm, Col>
where
    Col: Column,
    Col::WindowKind<'imm>: AssocWindowPull<'imm, ImmData, MutData>,
{
    type ImmPull = <Col::WindowKind<'imm> as AssocWindowPull<'imm, ImmData, MutData>>::ImmPull;

    fn pull(&mut self, key: Self::Key) -> Access<Self::ImmPull, MutData> {
        let index = self.gen.pull_key(key)?;
        Ok(Entry {
            index,
            data: unsafe { self.col.pull(index) },
        })
    }

    fn insert(&mut self, val: Data<ImmData, MutData>) -> (Self::Key, InsertAction) {
        let (key, action) = self.gen.insert();
        match &action {
            InsertAction::Place(ind) => unsafe { self.col.place(*ind, val) },
            InsertAction::Append => self.col.append(val),
        }
        (key, action)
    }
}
