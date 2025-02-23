pub mod own;
pub mod rc;
pub mod weak;

pub trait Arena<Data> {
    type Cfg;
    type Key;

    fn new(cfg: Self::Cfg) -> Self;
    fn insert(&mut self, data: Data) -> Option<Self::Key>;

    fn read(&self, key: &Self::Key) -> &Data;
    fn write(&mut self, key: &Self::Key) -> &mut Data;

    fn delete(&mut self, key: Self::Key);
}
