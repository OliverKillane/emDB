#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::linkedlist)]
#![allow(clippy::needless_pass_by_value)]

use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::TokenStream;
use proc_macro_error::{proc_macro_error, Diagnostic};

mod gen_match;
mod impl_trait;
mod macro_comm;
mod quick_enum;
mod quick_from;
mod store;

fn emit_result<ERR: IntoIterator<Item = Diagnostic>>(
    res: Result<TokenStream, ERR>,
) -> CompilerTokenStream {
    match res {
        Ok(out) => out,
        Err(es) => {
            for e in es {
                e.emit();
            }
            TokenStream::new()
        }
    }
    .into()
}

/// Stores an item's tokens to a macro store for later use.
/// - Can be applied to any item (functions, structs, enums, traits, etc.).
/// - Performs no transformations or checks on the item.
/// ```
/// # use enumtrait;
/// #[enumtrait::store(foo_macro_store)]
/// enum Foo {
///     Bar,
///     Bing,
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn store(attrs: CompilerTokenStream, item: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(store::interface(attrs.into(), item.into()))
}

/// Transforms an enum definition such that all variants are single field tuple structs.
/// - variants of kind `StructName,` are converted to `StructName(StructName)`
/// - variants of kind `VariantName(Type)` are not changed
/// - other variant kinds throw an error
/// ```
/// # use enumtrait;
/// struct Bar { bar_val: usize }
/// struct Bing { bing_val: usize }
///
/// #[enumtrait::quick_enum]
/// enum Foo<'a> {
///     Bar,
///     Bing,
///     Bosh(&'a str)
/// }
///
/// fn check(f: Foo) -> usize {
///     match f {
///         Foo::Bar(b) => b.bar_val,
///         Foo::Bing(b) => b.bing_val,
///         Foo::Bosh(s) => s.len(),
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn quick_enum(attrs: CompilerTokenStream, item: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(quick_enum::interface(attrs.into(), item.into()))
}

/// Generates a from implementation for every tuple variant of an enum with a single member.
/// - All other variants are ignored.
/// ```
/// # use enumtrait;
///
/// struct Bop { x: i32 };
/// struct Bing {j: i32};
/// struct Baz<T> {val: usize, assoc: T};
/// struct Bar;
///
/// #[enumtrait::quick_enum]
/// #[enumtrait::quick_from]
/// enum Foo<T> {
///     Bop,
///     Bing,
///     Baz(Baz<T>),
///     Bar,
/// }
///
/// fn check() {
///     let foo_0: Foo<i32> = Bop{x: 7}.into();
///     let foo_1: Foo<i32> = Bing{j: -32}.into();
///     let foo_2: Foo<i32> = Baz{val: 42, assoc: 3}.into();
///     let foo_3: Foo<i32> = Bar.into();
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn quick_from(attrs: CompilerTokenStream, item: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(quick_from::interface(attrs.into(), item.into()))
}

/// Generates a match expression for an enumeration
/// - The enum should be of the form generated by [`macro@quick_enum`]
/// - due to an unresolved issue (TODO) with macros in an expression context,
///   and trailing `;`, the macro_store provided must be an identifier (no `path::style::names`)
/// ```
/// # use enumtrait;
/// struct Bar { common_field: usize }
/// struct Bing { common_field: usize, other_field: String }
///
/// #[enumtrait::quick_enum]
/// #[enumtrait::store(foo_macro_store)]
/// enum Foo {
///     Bar,
///     Bing,
/// }
///
/// fn check(f: Foo) -> usize {
///     enumtrait::gen_match!(foo_macro_store as f for it => it.common_field)
/// }
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn gen_match(input: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(gen_match::interface(input.into()))
}

#[doc(hidden)]
#[proc_macro]
pub fn gen_match_apply(input: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(gen_match::apply(input.into()))
}

/// Generates a trait implementation for an enumeration to allow for polymorphism
/// without `dyn`
/// - The enum should be of the form generated by [`macro@quick_enum`]
/// - Only function items with a reciever (e.g. `fn foo (self) -> T;` in the trait are created, associated types and constants
///   can be supplied manually
/// - Generics are unmodified, so the generic arguments for the trait need to be named exactly
///   as they are for the trait's definition. Be careful here - this macro does not check.
/// ```
/// # use enumtrait;
/// # struct Bar { common_field: usize }
/// # struct Bing { common_field: usize, other_field: String }
/// // given some structs `Foo` and `Bing`
/// #[enumtrait::quick_enum]
/// #[enumtrait::store(foo_macro_store)]
/// enum Foo {
///     Bar,
///     Bing,
/// }
///
/// #[enumtrait::store(foo_trait_store)]
/// trait FooTrait {
///     const baz: usize;
///     fn foo(&self) -> usize;
/// }
///
/// # impl FooTrait for Bar {
/// #     const baz: usize = 2;  
/// #     fn foo(&self) -> usize { self.common_field }
/// # }
/// # impl FooTrait for Bing {  
/// #     const baz: usize = 2;
/// #     fn foo(&self) -> usize { self.common_field }
/// # }
/// // ...Implement `FooTrait` for `Bar` and `Bing`
///
/// #[enumtrait::impl_trait(foo_trait_store for foo_macro_store)]
/// impl FooTrait for Foo {
///     const baz: usize = 42;
/// }
///
/// fn check(f: Foo) -> usize {
///     f.foo()
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn impl_trait(attrs: CompilerTokenStream, item: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(impl_trait::interface(attrs.into(), item.into()))
}

#[doc(hidden)]
#[proc_macro_error]
#[proc_macro]
pub fn impl_trait_apply(input: CompilerTokenStream) -> CompilerTokenStream {
    emit_result(impl_trait::apply(input.into()))
}
