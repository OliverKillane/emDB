//! EmDB!

pub use emdb_core as macros;

/// The dependencies used by code generated by the emdb macros
/// - Needs to be exported from emdb so the library can support the code it
///   generates
/// - Cannot export both proc-macros and normal items from a proc-macro crate,
///   hence this separation between [`emdb`](crate) and [`emdb_core`](macros)
pub mod dependencies {
    pub use minister;
    pub use pulpit;
}
