#![allow(refining_impl_trait)] // for refining the return `impl Any` types from emql generated traits.
//! # Benchmarks for EmDB against DuckDB and SQLite
//! For each benchmark we use [`emdb`] to generate the implementation, as well as a trait we can
//! used to define the other implementations (allows us to write a single generic benchmark based
//! on that trait).
//! - Additional trait bounds can be specified using [emdb]'s `Interface` backend.
//! - All return types are unconstrained, so other implementations can return their
//!   own types
//!
//! ## Constraints
//! These are particularly slow in duckdb, and are not used to speed up joins (see
//! [duckdb constraints](https://duckdb.org/docs/guides/performance/schema#constraints)).

// The schema implementations
pub mod data_logs;
pub mod sales_analytics;
pub mod user_details;

pub mod utils;
