#![doc = include_str!("../README.md")]
//!
//! ## Examples of Compile-time Protection
//! Cannot reuse the same token for multiple arenas.
//! ```compile_fail,E0382
//! # use smart_arenas::prelude::*;
//! context(|tk1| {
//!     context(|tk2| {
//!         let mut arena1 = Own::<Key<u32>, Blocks<32>, u32>::new(0, tk1);
//!         let mut arena2 = Own::<Key<u32>, Blocks<32>, u32>::new(0, tk1);
//!     })
//! });
//! ```
//!
//! Cannot return tokens for use in other contexts.
//! ```compile_fail
//! # use smart_arenas::prelude::*;
//! let tk1 = context(|tk1| {
//!     tk1
//! });
//! ```
//!
//! Cannot use a key for one arena, on another (even with the same data type, key index size & allocator).
//! ```compile_fail,E0521
//! # use smart_arenas::prelude::*;
//! context(|tk1| {
//!     context(|tk2| {
//!         let mut arena1 = Own::<Key<u32>, Blocks<32>, usize>::new(0, tk1);
//!         let arena2 = Own::<Key<u32>, Blocks<32>, usize>::new(0, tk2);
//!         
//!         let key1 = arena1.insert(23).unwrap();
//!
//!         let _ = arena2.read(&key1);
//!     })
//! });
//! ```
//!
//! While a function may use parameters that type check (here two arenas with the same type), it is
//! impossible to construct arguments for this function.
//! ```compile_fail,E0521
//! # use smart_arenas::prelude::*;
//! fn access<'id>(
//!     arena1: impl Arena<'id, Key = Key<'id, u32>, Data = usize>,
//!     arena2: impl Arena<'id, Key = Key<'id, u32>, Data = usize>,
//!     key1: Key<'id, u32>,
//! ) {
//!     let _ = arena1.read(&key1);
//!     let _ = arena2.read(&key1);
//! }
//! context(|tk1| {
//!     context(|tk2| {
//!         let mut arena1 = Own::<Key<u32>, Blocks<32>, usize>::new(0, tk1);
//!         let arena2 = Own::<Key<u32>, Blocks<32>, usize>::new(0, tk2);
//!         let key1 = arena1.insert(23).unwrap();
//!         access(arena1, arena2, key1);
//!     })
//! });
//! ```
#![feature(phantom_variance_markers)]

pub mod alloc;
pub mod arena;
pub mod id;

pub mod prelude {
    pub use crate::alloc::*;
    pub use crate::arena::*;
    pub use crate::id::index::*;
    pub use crate::id::key::*;
    pub use crate::id::token::*;
    pub use crate::multiple_context;
}
