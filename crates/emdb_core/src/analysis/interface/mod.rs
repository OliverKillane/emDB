//! Generates the types and expressions for the user to interact through.
//! - generating closures for user provided expressions that capture
//!   [`crate::plan::Query`] parameters
//! - generating the type definitions to use in queries.
//!
//! ```ignore
//!
//! mod my_impl {
//!     type Scalar0 = ...;
//!     struct Record0 = ...;
//!     
//!     struct Table = ...;
//!
//!     pub struct DB {
//!         table1: Table,
//!         table2: Table,
//!     }
//!     
//!     impl DB {
//!          pub fn query_name(&self, params) -> () {
//!                 
//!          }
//!     }
//! }
//! ```

/// TODO: make this work with the TableGet types
pub mod asserts;
pub mod contexts;
pub mod names;
pub mod types;
