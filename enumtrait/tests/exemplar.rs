use enumtrait;

struct Foo;
struct Cool<'a, A> {
    inner: &'a A,
}
struct Bar;

#[enumtrait::get_enum(polynum)]
enum Poly<'a, T> {
    Foo,
    Bar,
    Alias(Foo),
    Outer(Cool<'a, T>),
}

#[enumtrait::get_trait(polynum => polytrait)]
trait PolyTrait<'a, 'b, Z> {
    fn bing(self, a: i32) -> &'b Z;
    fn bong<B>(&mut self, b: B) -> &'a Z;
}

#[enumtrait::impl_trait(polytrait)]
impl<'x, 'y, J> PolyTrait<Blah, J> for Poly<'x, J> {}
