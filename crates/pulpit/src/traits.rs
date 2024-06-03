//! # Traits for implementing pulpit tables
//! Required for writing generic code to interact with pulpit tables.

pub trait BasicTable {
    type Key;
    type Get;
    type Brw;
    type Insert;
    type KeyErr;
    fn get(&self, key: Self::Key) -> Result<Self::Get, Self::KeyErr>;
    fn brw(&self, key: Self::Key) -> Result<Self::Brw, Self::KeyErr>;
    fn scan(&self) -> impl Iterator<Item = Self::Key> + '_;
    fn count(&self) -> usize;
}

pub trait Insert: BasicTable {
    fn insert(&mut self, data: Self::Insert) -> Self::Key;
}

pub trait InsertErr: BasicTable {
    type InsertErr;
    fn insert(&mut self, data: Self::Insert) -> Result<Self::Key, Self::InsertErr>;
}

pub trait Update<UpdateKind>: BasicTable {
    type UpdateErr;
    fn update(&mut self, update: UpdateKind, key: Self::Key) -> Result<(), Self::UpdateErr>;
}

pub trait Delete: BasicTable {
    fn delete(&mut self, key: Self::Key) -> Result<(), Self::KeyErr>;
}
