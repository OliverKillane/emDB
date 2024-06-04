use super::*;

/// An adapter used as a primary column when associated columns are all that 
/// is needed.
pub struct PrimaryAppendAdapter {
    max_key: usize,
    /// Required as to fit the interface we need to be able to return `&mut ()`, 
    /// however we cannot do the neat lifetime extension trick of `&()` with `&mut` 
    mut_val: (),
}

impl Keyable for PrimaryAppendAdapter {
    type Key = usize;
}

impl Column for PrimaryAppendAdapter {
    type WindowKind<'imm> = Window<'imm, PrimaryAppendAdapter> where Self: 'imm;

    fn window(&mut self) -> Self::WindowKind<'_> {
        Window {
            inner: self,
        }
    }

    fn new(_: usize) -> Self {
        Self {
            max_key: 0,
            mut_val: (),
        }
    }
}

impl<'imm> PrimaryWindow<'imm, (), ()> for Window<'imm, PrimaryAppendAdapter>
{
    type ImmGet = ();
    type Col = PrimaryAppendAdapter;

    fn get(&self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmGet, ()> {
        if key < self.inner.max_key {
            Ok(Entry {
                index: key,
                data: Data{ imm_data: (), mut_data: () },
            })
        } else {
            Err(KeyError)
        }
    }

    fn brw(&self, key: <Self::Col as Keyable>::Key) -> Access<&(), &()> {
        if key < self.inner.max_key {
            Ok(Entry {
                index: key,
                data: Data { imm_data: &(), mut_data: &() },
            })
        } else {
            Err(KeyError)
        }
    }

    fn brw_mut(&mut self, key: <Self::Col as Keyable>::Key) -> Access<&(), &mut ()> {
        if key < self.inner.max_key {
            Ok(Entry {
                index: key,
                data: Data { imm_data: &(), mut_data: &mut self.inner.mut_val },
            })
        } else {
            Err(KeyError)
        }
    }

    fn conv_get(_: Self::ImmGet) -> () {
        ()
    }

    fn scan<'brw>(&'brw self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + 'brw {
        0..(self.inner.max_key)
    }

    fn count(&self) -> usize {
        self.inner.max_key
    }
}

impl<'imm> PrimaryWindowApp<'imm, (), ()> for Window<'imm, PrimaryAppendAdapter>
{
    fn append(&mut self, val: Data<(), ()>) -> <Self::Col as Keyable>::Key {
        let key = self.inner.max_key;
        self.inner.max_key += 1;
        key
    }

    unsafe fn unppend(&mut self) {
        self.inner.max_key -= 1;
    }
}
