//! # The emQL language frontend
//! ## What is [`emQL`](crate::emql!)?
//! [`emQL`](crate::emql!) is a dataflow based query language, designed to be easy
//! to understand & easily integrable with rust.
//!
//! It is partly inspired by [influxdb's flux language](https://awesome.influxdata.com/docs/part-2/introduction-to-flux/)
//! but is heavily restructed (no custom functions, only streams of data).
//!
//! ## emQL over SQL?
//! SQL is a natural choice for a new database because it is standard, and allows for easy
//! comparison against many other SQL supporting databases.
//!
//! However emDB's rust-embedded nature presents issues with this:
//! 1. embedding non-SQL syntax rust code (not SQL) makes existing SQL expressions redundant.
//! 2. integration of rust code with the database is advantaged by row reference types, which
//!    SQL does not support (unless through generated primary keys, accessed with a `WHERE id
//!    = id` clause).
//! 3. SQL allows for nullability by default, resulting in significant complexity to manage wrapping,
//!    and unwrapping, and the removal of some safety for unwraps.
//! 4. There is little overlap between SQL and rust syntax in keywords and types, resulting in
//!    poor syntax highlighting.
//!
//! Furthermore in general SQL is considered an unecessarily complex language, especially
//! when creating transactions that perform multiple actions, and need to share data.
//!
//! While none of these issues are insurmountable, it easier to develop a new, simplified language.
//! This would also allow me to alter syntax as needed to support improved error messages, and choose
//! a type system my code generation backend can most easily take advantage of.
//!
//! Using rust keywords and some syntax to trick rust-analyzer into providing coloured syntax
//! highlighting.
//!
//! A SQL-like implementation is still possible with the structure of emDB.
//!
//! ## Implementation
//! ### Parsing emQL from [`TokenStream`]
//! Due to the limitations of inter-macro communication, the entire schema (and quries) needed to be
//! defined in a single macro invocation.
//!
//! For parsing the emql language, the Combi library is used:
//! - Supports the LL(1) grammar required
//! - Allowed for easy error recovery, without needing a more complex AST with error nodes
//! - Produces rust Diagnostics, which can be emitted to rustc to be displayed as errors (and
//!   picked up by the language server for IDE support)
//! - good performance
//! - As part of the emDB project, can be tweaked and optimised as required for emDB's use
//!
//! ### [rustc API](https://rustc-dev-guide.rust-lang.org/rustc-driver.html) versus token passthrough
//! For embedded rust expressions there were two options for semantic analysis
//!
//! 1. Use the rust API to fully analyse passed expressions
//!    - provides the guarentee that all logical plans contain valid embedded rust
//!    - allows for type inference on expressions
//!    However it has a significant drawback:
//!    - Cannot analyse code from outside the macro invocation, so cannot use types
//!      and functions from outside [`emQL`](crate::emql!)
//!    - the API is exposed compiler library internals, so is subject to change, not ideal for a
//!      library that needs to work across compiler versions
//!
//! 2. Pass provided rust expressions through to the backend, and then onto the rust
//!    compiler unchecked
//!    - analysis is done on the final program, with context of all available types
//!      everywhere in the crate by rustc
//!    - reduces frontend complexity, and redundant work (analysing code twice, once
//!      in macro, once for end result)
//!    However
//!    - With no backend implementations, no expressions are checked
//!
//! I decided to check expression syntax (using syn) in the frontend and use the passthrough design.
//!  
//! One way to remedy this is to include a semantics generator as a backend, that forwards the
//! expressions to generated code purely to let rustc check.
//!
//! ## Potential Improvements
//! ### Types
//! Currently there are 3 main type syntaxes:
//! 1. `ref <table>` references a row
//! 2. `type <emdb type>` for referencing a type decided by emdb (e.g. a bag)
//! 3. A rust type (that cannot contain the two emdb types)
//!
//! To improve we could use our own type parser to parse rust types that can
//! include emdb types.
//! ```ignore
//! |> map(x: (ref my_table, Vec<(type bag, ref other_table)>) = ...)
//! ```
//!
//! ### Table References
//! Change semantic analysis to remove restriction on table reference types
//! being used in tables.
//!  - Should include a constraint to ensure that a reference is always valid
//!    (reference counting, check at end of transaction)
//!
//! ```ignore
//! table people {
//!     name: String,
//!     best_friend: ref people,
//! } @ [ valid_ref(best_friend) ]
//! ```
//!
//! ### Better ergonomics
//! Reducing the boilerplate required for the examples
//! - `update` being able to reference current values
//! - `deref` being easier to use in expressions
//!
//! ### Better Bag Support
//! Allow passing `type <emdb type>` into queries (using the types) from the
//! query.
//!
//! Implemented by evaluating the context, then analysing the types used for parameters.
//!
//! ```ignore
//! query foo(my_bag: type foo_bag) {
//!     use my_bag
//!         |> collect(foo_ret as type foo_bag) // defining the type to input
//!         ~> return;
//! }
//! ```

mod ast;
mod errors;
mod operators;
mod parse;
mod sem;
use crate::backend;
use std::collections::LinkedList;

use crate::{frontend::Frontend, plan};
use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;

pub struct Emql;

impl Frontend for Emql {
    fn from_tokens(
        input: TokenStream,
    ) -> Result<(plan::Plan, backend::Targets), LinkedList<Diagnostic>> {
        sem::ast_to_logical(parse::parse(input)?)
    }
}
