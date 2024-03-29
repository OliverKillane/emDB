use emdb::database;

// TODO: update for the removal of genpk

database! {
    impl graph as user_details_view;
    impl simple as user_details;

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
        pred(premium || credits > 0) as prem_credits
    ]

    // Description:
    //   Create a row, pipe to insert, insert returns gen_pk id
    // Reasoning:
    //   - Needed for data insert, generation of id only occurs from here,
    //     hence we know the table alone determines id
    //   - Move semantics (taking ownership of data structure from outside the database)
    query new_user(username: String, prem: bool) {
        row(name: String = username, premium: bool = prem, credits: i32 = 0 )
            ~> insert(users)
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
            |> collect
            ~> return;
    }

    // Description
    //   Update a given user's credits
    // Reasoning:
    //   - Need to apply constraint immediately
    //   - Need to index data structure
    //   - Database can see only credits is updated
    query add_credits(user: ref users, creds: i32) {
        row(user_id: ref users = users)
            ~> update(user_id use credits = credits + creds);
    }

    // Description:
    //   Apply multiplier bonus to premium users, and return the number of credits added
    // Reasoning:
    //   - Applying function over a tight loop
    //   - Iteration advantage form splitting premium users & non-premium
    //   - can be inlined to very simple iterate over &mut and increment sum
    query reward_premium(cred_bonus: f32) {
        ref users
            |> deref(users as it)
            |> filter(it.premium)
            |> map(users: ref users = users, new_creds: i32 = ((it.credits as f32) * cred_bonus) as i32)
            |> update(users use credits = new_creds)
            |> map(creds: i32 = new_creds)
            |> fold(sum: i64 = 0 -> sum + creds)
            ~> return;
    }

    // Description:
    //   Get the total number of credits in the premium table
    // Reasoning:
    //   Easy IVM case, all updates & inserts just need to add difference to
    //   the view
    query total_premium_credits() {
        use users
            |> filter(premium)
            |> map(credits: i64 = credits)
            |> fold(sum: i64 = 0 -> sum + credits)
            ~> return;
    }
}
