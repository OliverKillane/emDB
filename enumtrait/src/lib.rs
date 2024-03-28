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

#[proc_macro_attribute]
pub fn my_attribute_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input = parse_macro_input!(item as ItemFn);

    // Generate some new code to replace the original function
    let new_code = quote! {
        #input

        println!("This is a custom message from my_attribute_macro!");
    };

    // Return the new code as a TokenStream
    TokenStream::from(new_code)
}
