#![doc = include_str!("../README.md")]

pub mod alloc;
pub mod arenas;
pub mod utils;

pub mod prelude {
    pub use super::alloc::blocks::*;
    pub use super::alloc::contig::*;
    pub use super::alloc::*;

    pub use super::arenas::own::*;
    pub use super::arenas::strong::*;
    pub use super::arenas::*;

    pub use super::utils::unique::*;
}
