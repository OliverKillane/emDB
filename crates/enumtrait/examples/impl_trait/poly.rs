#![allow(unused_variables)]

struct Foo;
struct Cool<'a, A> {
    inner: &'a A,
}
struct Bar;

#[enumtrait::quick_enum]
#[enumtrait::store(poly_enum)]
enum Poly<'a, T> {
    Foo,
    Bar,
    Alias(Foo),
    Outer(Cool<'a, T>),
}

#[enumtrait::store(poly_trait)]
trait PolyTrait<'x, 'y, Z> {
    fn bing(self, a: i32) -> &'x Z;
    fn bong<B>(&mut self, b: Z) -> &'y B;
}

impl <'x, 'y, Z> PolyTrait<'x, 'y, Z> for Foo {
    fn bing(self, a: i32) -> &'x Z {
        unimplemented!()
    }
    fn bong<B>(&mut self, b: Z) -> &'y B {
        unimplemented!()
    }
}

impl <'x, 'y, Z> PolyTrait<'x, 'y, Z> for Bar {
    fn bing(self, a: i32) -> &'x Z {
        unimplemented!()
    }
    fn bong<B>(&mut self, b: Z) -> &'y B {
        unimplemented!()
    }
}

impl <'a, 'x, 'y, A, Z> PolyTrait<'x, 'y, Z> for Cool<'a, A> {
    fn bing(self, a: i32) -> &'x Z {
        unimplemented!()
    }
    fn bong<B>(&mut self, b: Z) -> &'y B {
        unimplemented!()
    }
}

#[enumtrait::impl_trait(poly_trait for poly_enum)]
impl<'x, 'y, Z> PolyTrait<'x, 'y, Z> for Poly<'y, Z> {}

fn main() {}
