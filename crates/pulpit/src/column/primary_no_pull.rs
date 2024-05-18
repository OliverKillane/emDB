use super::*;

/// Converts an [`AssocWindow`] (unchecked, without an index [`Column`]) into a
/// safely indexed [`Column`] that can be windowed into a  [`PrimaryWindow`].
pub struct PrimaryAppend<Col> {
    col: Col,
    max_key: usize,
}

impl <Col> Keyable for PrimaryAppend<Col> {
    type Key = usize;
}

impl<Col: Column> Column for PrimaryAppend<Col> {
    type WindowKind<'imm> = WindowPrimaryAppend<'imm, Col> where Self: 'imm;

    fn window(&mut self) -> Self::WindowKind<'_> {
        WindowPrimaryAppend {
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

pub struct WindowPrimaryAppend<'imm, Col: Column + 'imm> {
    col: Col::WindowKind<'imm>,
    max_key: &'imm mut usize,
}

impl<'imm, ImmData, MutData, Col> PrimaryWindow<'imm, ImmData, MutData>
    for WindowPrimaryAppend<'imm, Col>
where
    Col: Column,
    Col::WindowKind<'imm>: AssocWindow<'imm, ImmData, MutData>,
{
    type ImmGet = <Col::WindowKind<'imm> as AssocWindow<'imm, ImmData, MutData>>::ImmGet;
    type Col = PrimaryAppend<Col>;

    fn get(&self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmGet, MutData> {
        if key < *self.max_key {
            Ok(Entry {
                index: key,
                data: unsafe { self.col.get(key) },
            })
        } else {
            Err(KeyError)
        }
    }

    fn brw(&self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &MutData> {
        if key < *self.max_key {
            Ok(Entry {
                index: key,
                data: unsafe { self.col.brw(key) },
            })
        } else {
            Err(KeyError)
        }
    }

    fn brw_mut(&mut self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &mut MutData> {
        if key < *self.max_key {
            Ok(Entry {
                index: key,
                data: unsafe { self.col.brw_mut(key) },
            })
        } else {
            Err(KeyError)
        }
    }

    fn conv_get(get: Self::ImmGet) -> ImmData {
        Col::WindowKind::conv_get(get)
    }
    
    fn scan(&self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> {
        0..(*self.max_key)
    }
}

impl<'imm, ImmData, MutData, Col> PrimaryWindowApp<'imm, ImmData, MutData>
    for WindowPrimaryAppend<'imm, Col>
where
    Col: Column,
    Col::WindowKind<'imm>: AssocWindow<'imm, ImmData, MutData>,
{
    fn append(&mut self, val: Data<ImmData, MutData>) -> <Self::Col as Keyable>::Key {
        let key = *self.max_key;
        *self.max_key += 1;
        self.col.append(val);
        key
    }
    
    unsafe fn unppend(&mut self) {
        self.col.unppend();
    }
}
