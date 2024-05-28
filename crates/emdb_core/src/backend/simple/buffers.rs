//! The buffer structures to be passed through the backend.

use std::marker::PhantomData;

/// An ordered buffer of items.
pub trait Buffer<Inner> {
    fn push(&mut self, item: Inner);
    fn size_hint(&self) -> usize;
    fn scan_borrow<'a>(&'a self) -> impl Iterator<Item=&'a Inner> where Inner: 'a;
    fn scan_move(self) -> impl Iterator<Item=Inner>;
    fn clone(&self) -> Self;
}

/// A simple vector based buffer
pub struct VectorBuffer<Inner>(Vec<Inner>);
impl <Inner> Buffer<Inner> for VectorBuffer<Inner> where Inner: Clone {
    fn push(&mut self, item: Inner) {
        // TODO: Failable collections
        self.0.push(item)
    }

    fn size_hint(&self) -> usize {
        self.0.len()
    }

    fn scan_borrow<'a>(&'a self) -> impl Iterator<Item=&'a Inner> where Inner: 'a {
        self.0.iter()
    }

    fn scan_move(self) -> impl Iterator<Item=Inner> {
        self.0.into_iter()
    }
    
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl <Inner> From<Vec<Inner>> for VectorBuffer<Inner> {
    fn from(vec: Vec<Inner>) -> Self {
        VectorBuffer(vec)
    }
}

impl <Inner> FromIterator<Inner> for VectorBuffer<Inner> {
    fn from_iter<T: IntoIterator<Item = Inner>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// A buffer formed of two child buffers, useful for unions.
/// 
/// But we need `n` way unions? Why not `[NUM_BUFFERS; Buffer]`
/// - We want to use different buffer types
/// - We do not want to use `dyn`
pub struct BufferUnion<Inner, BuffA, BuffB> 
where BuffA: Buffer<Inner>, BuffB: Buffer<Inner>, Inner: Clone {
    first: BuffA,
    second: BuffB,
    inner: PhantomData<Inner>
}

impl <Inner, BuffA, BuffB> Buffer<Inner> for BufferUnion<Inner, BuffA, BuffB>
where BuffA: Buffer<Inner>, BuffB: Buffer<Inner>, Inner: Clone {
    fn push(&mut self, item: Inner) {
        self.second.push(item)
    }

    fn size_hint(&self) -> usize {
        self.first.size_hint() + self.second.size_hint()
    }

    fn scan_borrow<'a>(&'a self) -> impl Iterator<Item=&'a Inner> where Inner: 'a {
        self.first.scan_borrow().chain(self.second.scan_borrow())
    }

    fn scan_move(self) -> impl Iterator<Item=Inner> {
        self.first.scan_move().chain(self.second.scan_move())
    }
    
    fn clone(&self) -> Self {
        BufferUnion{first: self.first.clone(), second: self.second.clone(), inner: PhantomData}
    }
}

impl <Inner, BuffA, BuffB> From<(BuffA, BuffB)> for BufferUnion<Inner, BuffA, BuffB>
where BuffA: Buffer<Inner>, BuffB: Buffer<Inner>,  Inner: Clone {
    fn from((first, second): (BuffA, BuffB)) -> Self {
        BufferUnion{first, second, inner: PhantomData}
    }
}