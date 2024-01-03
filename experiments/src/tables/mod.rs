//! ## Requirements
//! Some schema and queries that are.
//! - beating the abilities of DBtoaster in a useful way
//! - look somewhat typical of some basic state
//! - avoid too much complexity in schema (e.g. lots of foreign key relations), and simple enough to diagram for report
//! - have several possible optimisations to choose from / justifications for discussion in report
//!
//! ## TCP-C & TCP-E
//! For OLTP workloads.
//! - Complex, lots e.g.joins, foreign key relations, etc.
//!
//! ## YCSB
//! For OLTP workloads, implement their benchmarking interface, then it can interact.
//! - interface does not specify future queries, queries implemented in each system's own lang (easy for SQL based, custom for non-sql)
//! - Workloads take parameters, then make calls to wrapper for database.
//! - Can find [core workloads here](https://github.com/brianfrankcooper/YCSB/tree/master/core/src/main/java/site/ycsb/workloads)
//! - Not just a schema & some queries as with TCP.
//!
//! I cannot entirely extract a basic query to reason about.
//!
//! ## Custom Schema
//! ### Schema
//! ```ignore
//! // mocked up in emQL frontend not yet complete
//! database! {
//!     name user_details;
//!
//!     // Reasoning:
//!     //  - Constraint checking required, needs to fail immediately (hybrid IVM)
//!     //  - premium is immutable, and iterated over. So we can maintain a view of
//!     //    two tables for premium & non-premium users
//!     //  - Very simple table
//!     table users {
//!         name: String,
//!         premium: bool,
//!         credits: i32
//!     } @ gen_id(id), pred(premium || credits >= 0) as PremCredits;
//!
//!     // Description:
//!     //   Create a row, pipe to insert, insert returns gen_pk id
//!     // Reasoning:
//!     //   - Needed for data insert, generation of id only occurs from here,
//!     //     hence we know the table alone determines id
//!     //   - Move semantics (taking ownership of data structure from outside the database)
//!     query new_user(username: String, prem: bool) {
//!         row(name = username, premium = prem, credits = 0 )
//!             |> insert(users)
//!             ~> return;
//!     }
//!
//!     // Description
//!     //   Get an individual user's data.
//!     // Reasoning:
//!     //   - Performance reliant on access to users data structure
//!     //     hence need to make a good choice of mapping (user id -> data) here.
//!     query get_info(user_id: users::ID) {
//!         users
//!             |> unique(id is user_id)
//!             ~> return;
//!     }
//!
//!     // Description:
//!     //    Get a snapshot of the entire users table state
//!     // Reasoning:
//!     //    - We can collect the database to a single structure decided by the compiler.
//!     //    - This can be radically sped up by removing copying of the string (no row deletions,
//!     //      immutable attribute, return reference bound to lifetime of database).
//!     //    - choosing a data structure for `users` table that is good for iteration
//!     query get_snapshot() {
//!         users |> collect() ~> return;
//!     }
//!
//!     // Description
//!     //   Update a given user's credits
//!     // Reasoning:
//!     //   - Need to apply constraint immediately
//!     //   - Need to index data structure
//!     //   - Database can see only credits is updated
//!     query add_credits(user: users::ID, creds: i32) {
//!         ref users
//!             |> unique(it.id is user)
//!             ~> update(it.credits = credits + creds);
//!     }
//!
//!     // Description:
//!     //   Apply multiplier bonus to premium users, and return the number of credits added
//!     // Reasoning:
//!     //   - Applying function over a tight loop
//!     //   - Iteration advantage form splitting premium users & non-premium
//!     //   - can be inlined to very simple iterate over &mut and increment sum
//!     query reward_premium(cred_bonus: f32) {
//!         ref users
//!             |> filter(it.premium)
//!             |> map(user: ref users = it, new_creds: i32 = ((it.credits as f32) * cred_bonus) as i32)
//!             |> update(it.credits = new_creds)
//!             |> map(creds: i32 = new_creds)
//!             |> sum()
//!             ~> return;
//!     }
//!
//!     // Description:
//!     //   Get the total number of credits in the premium table
//!     // Reasoning:
//!     //   Easy IVM case, all updates & inserts just need to add difference to
//!     //   the view
//!     query total_premium_credits() {
//!         users
//!             |> filter(premium)
//!             |> map(credits: i64 = credits)
//!             |> sum()
//!             ~> return;
//!     }
//! }
//!```
//!```
//! # use experiments::tables::UserDetails;
//! # use experiments::tables::naive::NaiveDatabase as UserDetailsDB;
//! fn foo() {
//!     let mut db = UserDetailsDB::new();
//!     let bob = db.new_user(String::from("bob"), true);
//!
//!     assert_eq!(db.get_info(bob).unwrap().credits, 0);
//!     db.add_credits(bob, 10).unwrap();
//!
//!     assert_eq!(db.get_info(bob).unwrap().credits, 10);
//!     assert_eq!(db.reward_premium(2f32).unwrap(), 20);
//!     assert_eq!(db.total_premium_credits(), 20);
//! }
//! ```
//! ### Implementations
//! #### Assuming any possible query
//! Hashtable for id index (supporting update & delete), generational arena allocator (for deletions) for contents. Scan over table for filters. Snapshot copies strings.
//!
//! #### With known queries
//! Separate arena allocators for premium & non-premium, total premium credits updated on insert/update, snapshot uses string references.
//!
//! *Neither consider concurrency, this is difficult enough as it is.*
#![doc=include_str!("../../docs/users_test_query.drawio.svg")]
//! *Note: Types are not shown, each connection has an associated `name -> type` mapping (e.g `id -> usize, name -> &str`)*
//!
//! Compare against naive implementation (both embedded, fair comparison), can also compare against postgres (very unfair but to demonstrate).
//!
//! ### Workload
//! Measure rate of operations per time unit.
//! - Against a certain table size (updates and queries)
//! - Cost of inserts as the table size increases

pub mod foresight;
pub mod naive;

#[derive(Debug)]
pub struct PremCreditsErr;
#[derive(Debug)]
pub struct IDNotFoundErr;
#[derive(Debug)]
pub enum AddCreditsErr {
    PremCredits,
    IDNotFound,
}

pub struct UserInfo<S, ID> {
    pub id: ID,
    pub name: S,
    pub premium: bool,
    pub credits: i32,
}

pub trait UserDetails<'a> {
    type UsersID: Copy;
    type ReturnString;

    fn new() -> Self;
    // For the prupose of easy access to the id needed for the other operations,
    // the id is returned and no constraints breaches are possible.
    fn new_user(&mut self, username: String, prem: bool) -> Self::UsersID;
    fn get_info(
        &self,
        user_id: Self::UsersID,
    ) -> Result<UserInfo<Self::ReturnString, Self::UsersID>, IDNotFoundErr>;
    fn get_snapshot(&self) -> Vec<UserInfo<Self::ReturnString, Self::UsersID>>;
    fn add_credits(&mut self, user: Self::UsersID, creds: i32) -> Result<(), AddCreditsErr>;
    fn reward_premium(&mut self, cred_bonus: f32) -> Result<i32, PremCreditsErr>;
    fn total_premium_credits(&self) -> i64;
}

#[cfg(test)]
mod tests {
    use crate::tables::{foresight::ForesightDatabase, naive::NaiveDatabase};

    use super::*;

    #[test]
    fn basic_intersts() {
        fn test<'a, T: UserDetails<'a>>() {
            let mut db = T::new();
            let bob = db.new_user(String::from("bob"), true);

            assert_eq!(db.get_info(bob).unwrap().credits, 0);
            db.add_credits(bob, 10).unwrap();

            assert_eq!(db.get_info(bob).unwrap().credits, 10);
            assert_eq!(db.reward_premium(2f32).unwrap(), 20);
            assert_eq!(db.total_premium_credits(), 20);
        }

        test::<NaiveDatabase>();
        test::<ForesightDatabase>();
    }
}
