//! ## Chew
//! A library for low-cost parsing of binary data.
//!  - eliminating bounds checks with compile time known bounds & access
//!  - support for arbitrary bit-sized data
//! 
//! ### A note on borrowing
//! Rather than providing an interface to borrow data, [data::Data] is copied 
//! out on access.
//!  - For small values, this is more efficient than borrowing
//!  - Enables access to unaligned (i.e. bit aligned) data
//!  - Allows the underlying implementation freedom (no support for borrows required)
//! 
//! The library provides traits for extension to other [cursor::Cursor] implementations.
//!  - e.g. a lazily loaded structure for larger than memory data
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

/*
take cursor with bound
access without
re-access with different bound
etc
*/

pub mod access;
pub mod cursor;
pub mod data;
pub mod utils;
