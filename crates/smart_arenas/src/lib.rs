//! Main TODO: Use lifetimes for tags instead of types & typemap
//!  - keep compile time
//!  - allow unit tests to run properly
//!  - allow leaking?
#![doc = include_str!("../README.md")]
#![feature(phantom_variance_markers)]

pub mod alloc;
pub mod arena;
pub mod key;

pub mod experiment;
