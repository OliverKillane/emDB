use super::*;

/// Converts an [`AssocWindow`] (unchecked, without an index [`Column`]) into a
/// safely indexed [`Column`] that can be windowed into a  [`PrimaryWindow`].
pub struct SimpleKey<Col> {
    col: Col,
    max_key: usize,
}

impl<Col: Column> Column for SimpleKey<Col> {
    type WindowKind<'imm> = SimpleKeyWindow<'imm, Col> where Self: 'imm;

    fn window(&mut self) -> Self::WindowKind<'_> {
        SimpleKeyWindow {
            col: self.col.window(),
            max_key: &mut self.max_key,
        }
    }

    fn new(size_hint: usize) -> Self {
        Self {
            col: Col::new(size_hint),
            max_key: 0,
        }
    }
}

pub struct SimpleKeyWindow<'imm, Col: Column + 'imm> {
    col: Col::WindowKind<'imm>,
    max_key: &'imm mut usize,
}

impl<'imm, ImmData, MutData, Col> PrimaryWindow<'imm, ImmData, MutData>
    for SimpleKeyWindow<'imm, Col>
where
    Col: Column,
    Col::WindowKind<'imm>: AssocWindow<'imm, ImmData, MutData>,
{
    type ImmGet = <Col::WindowKind<'imm> as AssocWindow<'imm, ImmData, MutData>>::ImmGet;
    type Key = usize;

    fn get(&self, key: Self::Key) -> Access<Self::ImmGet, MutData> {
        if key < *self.max_key {
            Ok(Entry {
                index: key,
                data: unsafe { self.col.get(key) },
            })
        } else {
            Err(KeyError)
        }
    }

    fn brw(&self, key: Self::Key) -> Access<&ImmData, &MutData> {
        if key < *self.max_key {
            Ok(Entry {
                index: key,
                data: unsafe { self.col.brw(key) },
            })
        } else {
            Err(KeyError)
        }
    }

    fn brw_mut(&mut self, key: Self::Key) -> Access<&ImmData, &mut MutData> {
        if key < *self.max_key {
            Ok(Entry {
                index: key,
                data: unsafe { self.col.brw_mut(key) },
            })
        } else {
            Err(KeyError)
        }
    }
}

impl<'imm, ImmData, MutData, Col> PrimaryWindowApp<'imm, ImmData, MutData>
    for SimpleKeyWindow<'imm, Col>
where
    Col: Column,
    Col::WindowKind<'imm>: AssocWindow<'imm, ImmData, MutData>,
{
    fn append(&mut self, val: Data<ImmData, MutData>) -> Self::Key {
        let key = *self.max_key;
        *self.max_key += 1;
        self.col.append(val);
        key
    }
}
