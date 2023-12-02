//! ## Description
//! Converting tables to heaps, and pushing up maps.
//!
//!
//! ## Code Example
//! ```text
//! use emDB::database;
//!
//! database!(
//!     name MyDatabase;
//!
//!     table foos = {
//!         id: smalltext[20],
//!         description: largetext,
//!     }
//!
//!     query new_foo(id: `&str`, descr: `&str`) = {
//!         foos
//!             <| insert(id = id, description = descr)
//!     }
//!
//!     query top_10() = {
//!         foos
//!             |> map(id = id, descr = description, descr_len = len(description))
//!             |> order_by(descr_len, asc)
//!             |> limit(10)
//!             |> map(id = id, descr = descr_len)
//!             |> return;
//!     }
//! )
//! ```
//!
//! ### Information Advantage
//! - We only need to top 10, so can keep a heap.
//! - We can return references from `top_10` bound to the lifetime of the database safely (never updated, never removed)
//! - We can contiguously allocate all the descriptions (never updated or removed)
//! - The heap can have a fixed size of 10, as we only insert & never delete, we can ignore any values we dont need to store.
//!
//! ### To Optimise
//! We apply rule based optimisation on the whole plan (all queries & tables) to push the table (`foos`) through operators.
//! - Some pushes add constraints (e.g push through `order_by` constrains to the ordered data structures available, `limit` constrains to the fixed size data structures).
//! - We can apply other optimisations (e.g dead column removal)
#![doc=include_str!("plan.drawio.svg")]
//!
//! ### Generated Code
//! Ideally in this situation:
//! - `foos` is a fixed-size heap of `(&id, descr_len)`
//! - `foos[id]` is allocated in a single buffer, with an overflow buffer for larger than 20 characters.
//!
//! The concurrent interactions are between:
//! - single row insert from `new_foo`
//! - multi-row read from `top_10` in a single transaction
//!
//! As we have simplified to simple access:
//! - insert lock: can do fine-grained heap insert
//! - need to lock out for reading all.
//!
//! Potentially best solution is a coarse RW lock.

struct Top10Row<'a> {
    id: &'a str,
    descr: usize,
}

trait MyDatabase<'a> {
    type NewFooError;
    type Top10Error;
    fn new() -> Self;
    fn new_foo(&mut self, id: &str, descr: &str) -> Result<(), Self::NewFooError>;
    fn top_10(&'a self) -> Result<Vec<Top10Row<'a>>, Self::Top10Error>;
}

mod basic_impl {
    use typed_generational_arena::Arena;

    struct FoosRow {
        id: String,
        description: String,
    }

    pub struct MyDatabase {
        foos: Arena<FoosRow>,
    }

    impl<'a> super::MyDatabase<'a> for MyDatabase {
        type NewFooError = ();
        type Top10Error = ();

        fn new() -> Self {
            Self { foos: Arena::new() }
        }

        fn new_foo(&mut self, id: &str, descr: &str) -> Result<(), Self::NewFooError> {
            self.foos.insert(FoosRow {
                id: String::from(id),
                description: String::from(descr),
            });
            Ok(())
        }

        fn top_10(&'a self) -> Result<Vec<super::Top10Row<'a>>, Self::Top10Error> {
            let mut x = self
                .foos
                .iter()
                .map(|(_, row)| super::Top10Row {
                    id: &row.id,
                    descr: row.description.len(),
                })
                .collect::<Vec<_>>();
            x.sort_by(|a, b| a.descr.cmp(&b.descr));
            x.truncate(10);
            Ok(x)
        }
    }
}

mod heap;
mod optimised_impl {
    use super::*;
}
