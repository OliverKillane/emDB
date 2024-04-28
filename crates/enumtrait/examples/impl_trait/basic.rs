
struct Bar { common_field: usize }
struct Bing { common_field: usize, other_field: String }
// given some structs `Foo` and `Bing`
#[enumtrait::quick_enum]
#[enumtrait::store(foo_macro_store)]
enum Foo {
    Bar,
    Bing,
}

#[enumtrait::store(foo_trait_store)]
trait FooTrait {
    const BAZ: usize;
    fn foo(&self) -> usize;
}

impl FooTrait for Bar {  
    const BAZ: usize = 1;
    fn foo(&self) -> usize { self.common_field }
}
impl FooTrait for Bing { 
    const BAZ: usize = 2; 
    fn foo(&self) -> usize { self.common_field }
}

#[enumtrait::impl_trait(foo_trait_store for foo_macro_store)]
impl FooTrait for Foo {
    const BAZ: usize = 42;
}

fn check(f: Foo) -> usize {
    f.foo()
}
fn main() {}