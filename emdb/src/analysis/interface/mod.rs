//! Generates the types and expressions for the user to interact through.
//! - generating closures for user provided expressions that capture
//!   [`crate::plan::Query`] parameters
//! - generating the type definitions to use in queries.

pub mod asserts;
pub mod contexts;
pub mod names;
pub mod types;
