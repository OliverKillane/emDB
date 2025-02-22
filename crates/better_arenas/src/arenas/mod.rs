pub mod rc;
pub mod own;
pub mod weak;

pub trait Arena<Data> {
    type Cfg;
    type Key;

    fn new(cfg: Self::Cfg) -> Self;
    fn insert(&mut self, data: Data) -> Self::Key;

    fn read(&self, idx: &Self::Key) -> &Data;
    fn write(&self, idx: &Self::Key) -> &mut Data;

    fn delete(&mut self, idx: Self::Key);
}
