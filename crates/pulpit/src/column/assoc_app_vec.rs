use super::*;

pub struct AssocAppVec<ImmData, MutData> {
    data: Vec<Data<ImmData, MutData>>,
}

impl<ImmData, MutData> Keyable for AssocAppVec<ImmData, MutData> {
    type Key = usize;
}

impl<ImmData, MutData> Column for AssocAppVec<ImmData, MutData> {
    type WindowKind<'imm> = Window<'imm, Self>
    where
        Self: 'imm;

    fn new(size_hint: usize) -> Self {
        AssocAppVec {
            data: Vec::with_capacity(size_hint),
        }
    }

    fn window(&mut self) -> Self::WindowKind<'_> {
        Window { inner: self }
    }
}

impl<'imm, ImmData, MutData> AssocWindow<'imm, ImmData, MutData>
    for Window<'imm, AssocAppVec<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmGet = ImmData;

    #[inline(always)]
    unsafe fn assoc_get(&self, ind: UnsafeIndex) -> Data<Self::ImmGet, &MutData> {
        let Data { imm_data, mut_data } = self.assoc_brw(ind);
        Data {
            imm_data: imm_data.clone(),
            mut_data,
        }
    }

    #[inline(always)]
    unsafe fn assoc_brw(&self, ind: UnsafeIndex) -> Data<&ImmData, &MutData> {
        let Data { imm_data, mut_data } = self.inner.data.get_unchecked(ind);
        Data { imm_data, mut_data }
    }

    #[inline(always)]
    unsafe fn assoc_brw_mut(&mut self, ind: UnsafeIndex) -> Data<&ImmData, &mut MutData> {
        let Data { imm_data, mut_data } = self.inner.data.get_unchecked_mut(ind);
        Data { imm_data, mut_data }
    }

    #[inline(always)]
    fn assoc_append(&mut self, val: Data<ImmData, MutData>) {
        self.inner.data.push(val)
    }

    #[inline(always)]
    fn conv_get(get: Self::ImmGet) -> ImmData {
        get
    }

    #[inline(always)]
    unsafe fn assoc_unppend(&mut self) {
        self.inner.data.pop();
    }
}

impl<'imm, ImmData, MutData> PrimaryWindow<'imm, ImmData, MutData>
    for Window<'imm, AssocAppVec<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    type ImmGet = ImmData;

    type Col = AssocAppVec<ImmData, MutData>;

    fn get(&self, key: <Self::Col as Keyable>::Key) -> Access<Self::ImmGet, &MutData> {
        if let Some(Data { imm_data, mut_data }) = self.inner.data.get(key) {
            Ok(Entry {
                index: key,
                data: Data {
                    imm_data: imm_data.clone(),
                    mut_data,
                },
            })
        } else {
            Err(KeyError)
        }
    }

    fn brw(&self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &MutData> {
        if let Some(Data { imm_data, mut_data }) = self.inner.data.get(key) {
            Ok(Entry {
                index: key,
                data: Data { imm_data, mut_data },
            })
        } else {
            Err(KeyError)
        }
    }

    fn brw_mut(&mut self, key: <Self::Col as Keyable>::Key) -> Access<&ImmData, &mut MutData> {
        if let Some(Data { imm_data, mut_data }) = self.inner.data.get_mut(key) {
            Ok(Entry {
                index: key,
                data: Data { imm_data, mut_data },
            })
        } else {
            Err(KeyError)
        }
    }

    fn conv_get(get: Self::ImmGet) -> ImmData {
        get
    }

    fn scan_brw<'brw>(&'brw self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + 'brw {
        self.scan_get()
    }

    fn scan_get(&self) -> impl Iterator<Item = <Self::Col as Keyable>::Key> + 'static {
        0..(self.inner.data.len())
    }

    fn count(&self) -> usize {
        self.inner.data.len()
    }
}

impl<'imm, ImmData, MutData> PrimaryWindowApp<'imm, ImmData, MutData>
    for Window<'imm, AssocAppVec<ImmData, MutData>>
where
    ImmData: Clone,
    MutData: Clone,
{
    fn append(&mut self, val: Data<ImmData, MutData>) -> <Self::Col as Keyable>::Key {
        let key = self.inner.data.len();
        self.inner.data.push(val);
        key
    }

    unsafe fn unppend(&mut self) {
        self.inner.data.pop();
    }
}
