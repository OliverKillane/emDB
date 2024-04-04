//! # Monotrait
//! A crate for deriving enum based polymorphism.
//!
//! ## Pattern
//! Often we want to implement traits separately for each variant of a enum.
//! This is traditionally implemented in two ways:
//!
//! ```
//! trait Bing {
//!     fn bonk(&self);
//! }
//!  
//! struct Foo();
//! impl Bing for Foo {
//!     fn bonk(&self) {}
//! }
//!  
//! struct Bar();
//! impl Bing for Bar {
//!     fn bonk(&self) {}
//! }
//!
//! fn method_1() {
//!     // using runtime polymorphism, at cost
//!     let bings: Vec<Box<dyn Bing>> = vec![Box::new(Foo()), Box::new(Bar())];
//!     for b in bings {
//!         b.bonk()
//!     }
//! }
//!
//! fn method_2() {
//!     // using an enum, at the cost of boilerplate
//!     enum BingVars {
//!         Foo(Foo),
//!         Bar(Bar),
//!     }
//!     
//!     impl Bing for BingVars {
//!         fn bonk(&self) {
//!             match self {
//!                 BingVars::Foo(i) => i.bonk(),
//!                 BingVars::Bar(i) => i.bonk(),
//!             }
//!         }
//!     }
//!
//!     let bings: Vec<BingVars> = vec![BingVars::Foo(Foo()), BingVars::Bar(Bar())];
//!     for b in bings {
//!         b.bonk()
//!     }
//! }
//!
//! fn main() {
//!     method_1();
//!     method_2();
//! }
//! ```
//!
//! The crate removes the boilerplate from `method_2` by generating the enum and the implementation for you.

/*
#[enumtrait::register]
enum Foo {
    Var1,
    Var2,
}

// now that we can invoke macros in attribute macros
#[enumtrait::implement(Foo_fields!())]
trait Bing {
    fn bonk(&self);
}

*/

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::proc_macro_error;

mod inner;

/// Registers an enum as an enumtrait
#[proc_macro_error]
#[proc_macro_attribute]
pub fn register(attr: TokenStream, item: TokenStream) -> TokenStream {
    match inner::register(TokenStream2::from(attr), TokenStream2::from(item)) {
        Ok(t) => t.into(),
        Err(es) => {
            for e in es {
                e.emit()
            }
            TokenStream::new()
        }
    }
}

#[proc_macro_error]
#[proc_macro]
pub fn implement(input: TokenStream) -> TokenStream {
    match inner::implement(TokenStream2::from(input)) {
        Ok(t) => t.into(),
        Err(es) => {
            for e in es {
                e.emit()
            }
            TokenStream::new()
        }
    }
}

macro_rules! interface_macro {
    ($p:path as $name:ident) => {
        #[proc_macro_attribute]
        fn $name(attr: TokenStream, item: TokenStream) -> TokenStream {
            $p (TokenStream2::from(attr), TokenStream2::from(item)).into()
        }
    }
}