use emdb::macros::emql;
use userdetails::Database;

emql! {
    impl userdetails as Interface{
        pub = on,
        traits_with_db = {super::UserDetailsWrap,},
    };
    impl emdb_impl as Serialized{
        interface = userdetails,
        pub = on,
    };

    // Reasoning:
    //  - Constraint checking required, needs to fail immediately (hybrid IVM)
    //  - premium is immutable, and iterated over. So we can maintain a view of
    //    two tables for premium & non-premium users
    //  - Very simple table
    table users {
        name: String,
        premium: bool,
        credits: i32,
    } @ [
        pred(*premium || *credits > 0) as prem_credits
    ]

    // Description:
    //   Create a row, pipe to insert, insert returns gen_pk id
    // Reasoning:
    //   - Needed for data insert, generation of id only occurs from here,
    //     hence we know the table alone determines id
    //   - Move semantics (taking ownership of data structure from outside the database)
    query new_user(username: String, prem: bool, start_creds: Option<i32>) {
        row(name: String = username, premium: bool = prem, credits: i32 = start_creds.unwrap_or(0) )
            ~> insert(users as ref user_id)
            ~> return;
    }

    // Description
    //   Get an individual user's data.
    // Reasoning:
    //   - Performance reliant on access to users data structure
    //     hence need to make a good choice of mapping (user id -> data) here.
    query get_info(user_id: ref users) {
        row(it: ref users = user_id)
            ~> deref(it as userdata)
            ~> return;
    }

     // Description:
    //    Get a snapshot of the entire users table state
    // Reasoning:
    //    - We can collect the database to a single structure decided by the compiler.
    //    - This can be radically sped up by removing copying of the string (no row deletions,
    //      immutable attribute, return reference bound to lifetime of database).
    //    - choosing a data structure for `users` table that is good for iteration
    query get_snapshot() {
        use users
            |> collect(it as type user_t)
            ~> return;
    }

    // Description
    //   Update a given user's credits
    // Reasoning:
    //   - Need to apply constraint immediately
    //   - Need to index data structure
    //   - Database can see only credits is updated
    query add_credits(user: ref users, creds: i32) {
        row(user_id: ref users = user)
            ~> deref(user_id as user)
            ~> update(user_id use credits = user.credits + creds);
    }

    // Description:
    //   Apply multiplier bonus to premium users, and return the number of credits added
    // Reasoning:
    //   - Applying function over a tight loop
    //   - Iteration advantage form splitting premium users & non-premium
    //   - can be inlined to very simple iterate over &mut and increment sum
    query reward_premium(cred_bonus: f32) {
        ref users as users_ref
            |> deref(users_ref as it)
            |> filter(*it.premium)
            |> map(users_ref: ref users = users_ref, new_creds: i32 = ((it.credits as f32) * cred_bonus) as i32)
            |> update(users_ref use credits = new_creds)
            |> map(creds: i32 = new_creds)
            |> fold(sum: i64 = 0 -> sum + creds as i64)
            ~> return;
    }

    // Description:
    //   Get the total number of credits in the premium table
    // Reasoning:
    //   Easy IVM case, all updates & inserts just need to add difference to
    //   the view
    query total_premium_credits() {
        use users
            |> filter(**premium)
            |> map(credits: i64 = credits as i64)
            |> fold(sum: i64 = 0 -> sum + credits)
            ~> return;
    }
}

// Required to get new user keys for other queries
pub trait UserDetailsWrap<'imm>: Database<'imm>  {
    fn new_user_wrap(&mut self, username: String, prem: bool, start_creds: Option<i32>) -> <Self::Datastore as userdetails::Datastore>::users_key;
}

impl <'imm>  UserDetailsWrap<'imm> for emdb_impl::Database<'imm> {
    fn new_user_wrap(&mut self, username: String, prem: bool, start_creds: Option<i32>) -> <Self::Datastore as userdetails::Datastore>::users_key {
        self.new_user(username, prem, start_creds).unwrap().user_id
    }
}

pub mod duckdb_impl;
pub mod sqlite_impl;