use enumtrait;

struct Bar {
    cool: i32,
}

struct Foo {
    zig: Bar,
}

#[enumtrait::register]
enum Baz {
    Foo,
    Bar,
}

Baz_enumitem! {
    Cool
}
