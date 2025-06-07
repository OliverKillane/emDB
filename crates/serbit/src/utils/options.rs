
// TODO(oliverkillane): Switch to a smallvec?
/// An append only structure of items.
///  - Always has at least one item.
pub struct Options<I> {
    first: I,
    additional: Vec<I>,
}

impl<I> Options<I> {
    pub fn new(first: I) -> Self {
        Self {
            first,
            additional: Vec::new(),
        }
    }

    pub fn push(&mut self, item: I) {
        self.additional.push(item);
    }

    pub fn get(&self) -> (&I, impl Iterator<Item = &I>) {
        let first = &self.first;
        let additional = self.additional.iter();
        (first, additional)
    }
}

impl<I> IntoIterator for Options<I> {
    type Item = I;
    type IntoIter = std::iter::Chain<std::iter::Once<I>, std::vec::IntoIter<I>>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.first).chain(self.additional.into_iter())
    }
}

impl<'brw, I> IntoIterator for &'brw Options<I> {
    type Item = &'brw I;
    type IntoIter = std::iter::Chain<std::iter::Once<&'brw I>, std::slice::Iter<'brw, I>>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(&self.first).chain(self.additional.iter())
    }
}
