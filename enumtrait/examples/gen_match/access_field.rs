
struct Bar { common_field: usize }
struct Bing { common_field: usize, other_field: String }

#[enumtrait::quick_enum]
#[enumtrait::store(foo_macro_store)]
enum Foo {
    Bar,
    Bing,
}

fn check(f: Foo) -> usize {
    enumtrait::gen_match!(foo_macro_store as f for it => it.common_field) + 3
}

fn main() {}