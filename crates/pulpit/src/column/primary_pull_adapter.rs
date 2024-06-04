use std::marker::PhantomData;

use super::*;

#[derive(Clone, Copy)]
enum GenEntry {
    Generation(usize),
    Hidden(usize),
    NextFree(Option<usize>),
}

struct GenInfo {
    next_free: Option<usize>,
    gen_counter: usize,
    generations: Vec<GenEntry>,
    visible_count: usize,
}

impl GenInfo {
    fn lookup_key<Store>(&self, key: GenKey<Store, usize>) -> Result<UnsafeIndex, KeyError> {
        match self.generations.get(key.index) {
            Some(GenEntry::Generation(g)) if key.generation == *g => Ok(key.index),
            _ => Err(KeyError),
        }
    }

    fn pull_key<Store>(&mut self, key: GenKey<Store, usize>) -> Result<UnsafeIndex, KeyError> {
        if let Some(entry) = self.generations.get_mut(key.index) {
            if let GenEntry::Generation(_) = entry {
                self.visible_count -= 1;
            }
            match *entry {
                GenEntry::Generation(g) | GenEntry::Hidden(g) if g == key.generation => {
                    *entry = GenEntry::NextFree(self.next_free);
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

    fn hide_key<Store>(&mut self, key: GenKey<Store, usize>) -> Result<(), KeyError> {
        if let Some(entry) = self.generations.get_mut(key.index) {
            match *entry {
                GenEntry::Generation(g) if g == key.generation => {
                    *entry = GenEntry::Hidden(g);
                    self.visible_count -= 1;
                    Ok(())
                }
                _ => Err(KeyError),
            }
        } else {
            Err(KeyError)
        }
    }

    fn reveal_key<Store>(&mut self, key: GenKey<Store, usize>) -> Result<(), KeyError> {
        if let Some(entry) = self.generations.get_mut(key.index) {
            match *entry {
                GenEntry::Hidden(g) if g == key.generation => {
                    *entry = GenEntry::Generation(g);
                    self.visible_count += 1;
                    Ok(())
                }
                _ => Err(KeyError),
            }
        } else {
            Err(KeyError)
        }
    }

    fn scan<Store>(&self) -> impl Iterator<Item = GenKey<Store, usize>> + '_ {
        self.generations
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match e {
                GenEntry::Generation(g) => Some(GenKey {
                    index: i,
                    generation: *g,
                    phantom: PhantomData,
                }),
                GenEntry::NextFree(_) | GenEntry::Hidden(_) => None,
            })
    }

    fn insert<Store>(&mut self) -> (GenKey<Store, usize>, InsertAction) {
        if let Some(k) = self.next_free {
            // TODO: could use unchecked here
            let entry = self.generations.get_mut(k).unwrap();
            match *entry {
                GenEntry::NextFree(opt) => {
                    self.next_free = opt;
                    *entry = GenEntry::Generation(self.gen_counter);
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
                .push(GenEntry::Generation(self.gen_counter));
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

    fn count(&self) -> usize {
        self.visible_count
    }
}

/// An adapter to allow for associated columns to be used with a primary.
/// - Used as the primary column for a table, but with only generation data 
///   (no user data)
pub struct PrimaryPullAdapter {
    gen: GenInfo,
    /// Required as to fit the interface we need to be able to return `&mut ()`, 
    /// however we cannot do the neat lifetime extension trick of `&()` with `&mut` 
    mut_val: (),
}

impl Keyable for PrimaryPullAdapter {
    type Key = GenKey<PrimaryPullAdapter, usize>;
}

impl Column for PrimaryPullAdapter {
    type WindowKind<'imm> =  Window<'imm, PrimaryPullAdapter> where Self: 'imm;

    fn new(size_hint: usize) -> Self {
        PrimaryPullAdapter {
            gen: GenInfo {
                next_free: None,
                generations: Vec::with_capacity(size_hint),
                gen_counter: 0,
                visible_count: 0,
            },
            mut_val: (),
        }
    }

    fn window(&mut self) -> Self::WindowKind<'_> {
        Window {
            inner: self,
        }
    }
}

impl<'imm> PrimaryWindow<'imm, (), ()> for Window<'imm, PrimaryPullAdapter>
{
    type ImmGet = ();
    type Col = PrimaryPullAdapter;

    fn get(&self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmGet, ()> {
        let index = self.inner.gen.lookup_key(key)?;
        Ok(Entry {
            index,
            data: Data { imm_data: (), mut_data: () },
        })
    }

    fn brw(&self, key: <Self::Col as Keyable>::Key) -> Access<&(), &()> {
        let index = self.inner.gen.lookup_key(key)?;
        Ok(Entry {
            index,
            data: Data{ imm_data: &(), mut_data: &() },
        })
    }

    fn brw_mut(&mut self, key: <Self::Col as Keyable>::Key) -> Access<&(), &mut ()> {
        let index = self.inner.gen.lookup_key(key)?;
        Ok(Entry {
            index,
            data: Data { imm_data: &(), mut_data: &mut self.inner.mut_val },
        })
    }

    fn conv_get(_: Self::ImmGet) -> () {
        ()
    }

    fn scan<'brw>(&'brw self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + 'brw {
        self.inner.gen.scan()
    }

    fn count(&self) -> usize {
        self.inner.gen.count()
    }
}

impl<'imm> PrimaryWindowPull<'imm, (), ()> for Window<'imm, PrimaryPullAdapter>
{
    type ImmPull = ();

    fn pull(&mut self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmPull, ()> {
        let index = self.inner.gen.pull_key(key)?;
        Ok(Entry {
            index,
            data: Data { imm_data: (), mut_data: () },
        })
    }

    fn insert(
        &mut self,
        _: Data<(), ()>,
    ) -> (<Self::Col as Keyable>::Key, InsertAction) {
        self.inner.gen.insert()
    }

    fn conv_pull(_: Self::ImmPull) -> () {
        ()
    }
}

impl<'imm> PrimaryWindowHide<'imm, (), ()> for Window<'imm, PrimaryPullAdapter>
{
    fn hide(&mut self, key: <Self::Col as Keyable>::Key) -> Result<(), KeyError> {
        self.inner.gen.hide_key(key)
    }

    fn reveal(&mut self, key: <Self::Col as Keyable>::Key) -> Result<(), KeyError> {
        self.inner.gen.reveal_key(key)
    }
}
