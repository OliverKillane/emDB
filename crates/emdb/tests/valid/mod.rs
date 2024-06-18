#![allow(dead_code, unused_variables)]
//! ## Valid Tests for emql interface
//! - Code is added in submodules from here, to be executed by the 
//!   [emql.rs](./../emql.rs) integration test
//! 
//! NOTE: *Cargo compiles each `.rs` file in the top level of the `tests/` 
//!       directory as a separate crate. subdirectories with modules are not 
//!       compiled as tests, but are available for the test crates to use.

pub mod complex;
pub mod context;
pub mod extreme;
pub mod simple;
pub mod fixme;