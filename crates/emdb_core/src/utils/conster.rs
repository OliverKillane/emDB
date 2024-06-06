//! # Compile time values without full const generics.
//!
//! ## Current Compile Time
//! Current state of const generics only allows for a limited set of integral types (bool, char,
//! signed & unsigned integer types). This 'min const generics' was introduced to appease
//! developer needs without needing to stabilise the more complex generics.
//!
//! In C++ this has already been partially solved, with support for non-type template parameters
//! of integral types, pointers to objects with static storage duration (equivalently in rust
//! `&'static`) though this can be somewhat cirtumvented with parameter packs of bytes and a
//! serialize/derserialize step (see [Template non-type arguments](https://en.cppreference.com/w/cpp/language/template_parameters))
//!
//! In Zig this has been elegantly solved by allowing `comptime` types, thus avoiding the need for a
//! separate compile time programming environment as with rust (const) and C++ (constexpr,
//! consteval, and templates)
//!
//! ## Future Compile Time
//! Several nightly features implement the unstabilised full const generics ([see this blog post](https://blog.rust-lang.org/inside-rust/2021/09/06/Splitting-const-generics.html))
//! - far from stabilised (for example floats at compile time, const in traits (combined with keyword
//!   generics from async this implies significant design decisions need to be made), interactions
//!   with macros)
//! - bugs exist in the current nightly implementations
//! - never ending big-macro vs comptime reflection debate +  macro system is in the process of
//!   being overhauled (e.g Diagnostics api, `macro` keyword for macro-rules - easier macros means less
//!   need for comptime)
//!
//! ## This Workaround
//! This module contains my tools for working around the current limitations.
#![allow(unused_macros, unused_imports)]

/// Associates values with types through the [Const] trait.
///
/// ```ignore
/// # use crate::utils::conster::conster;
/// conster!(const World: &'static str = "world");
///
/// const fn greet<C: Const<&'static str>>() -> fn(&str) -> String {
///     |greeting| {
///         let p = C::val();
///         format!("{greeting} {p}")
///     }
/// }
///
/// assert_eq!(greet::<World>()("hello"), String::from("hello world"));
/// ```
///
/// Multiple items are also allowed
/// ```ignore
/// struct Foo(String, [i32;3])
///
/// conster! {
///     const Foobar: Foo = Foo(String::from("hello"), [1,2,3]);
///     const Bosch: Foo = Foo(String::from("goodbye"), [4,5,6]);
///     const Cringe: &'static str = "cringe";
/// }
/// ```
///
/// *Sidenote: using a method, rather than an associated const in the [Const]
/// trait as there are more restrictions on const types*
macro_rules! conster {
    ($(const $name:ident: $data_type:ty = $val:expr);*) => {
        $(
            struct $name;
            impl Const<$data_type> for $name {
                fn val() -> $data_type { $val }
            }
        )*
    };
}
pub(crate) use conster;

pub trait Const<T> {
    fn val() -> T;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Foo(String, [i32; 3]);

    conster! {
        const World: &'static str = "world";
        const Bob: &'static str = "bob";
        const Eve: &'static str = "bob"; // (comedy: https://en.wikipedia.org/wiki/Alice_and_Bob)
        const Bosch: Foo = Foo(String::from("goodbye"), [4,5,6])
    }

    const fn greet<C: Const<&'static str>>() -> fn(&str) -> String {
        |greeting| {
            let p = C::val();
            format!("{greeting} {p}")
        }
    }

    #[test]
    fn basic_example() {
        assert_eq!(greet::<World>()("hello"), String::from("hello world"));
    }
}
