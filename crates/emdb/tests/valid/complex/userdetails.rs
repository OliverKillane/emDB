use emdb::macros::emql;
use user_deets::{Datastore, Database};

emql! {
    impl user_deets as Interface{
        pub = on,
    };
    impl my_db as Serialized{
        interface = user_deets,
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

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    let bob = db
        .new_user(String::from("Bob"), false, Some(3))
        .expect("empty database")
        .user_id;

    let alice = db
        .new_user(String::from("Alice"), true, None)
        .expect("empty database")
        .user_id;

    let bob_info = db.get_info(bob).unwrap();
    let alice_info = db.get_info(alice).unwrap();

    assert_eq!(bob_info.userdata.name, "Bob");
    assert_eq!(bob_info.userdata.premium, &false);
    assert_eq!(bob_info.userdata.credits, 3);

    assert_eq!(alice_info.userdata.name, "Alice");
    assert_eq!(alice_info.userdata.premium, &true);
    assert_eq!(alice_info.userdata.credits, 0);

    db.add_credits(bob, 10).unwrap();
    db.add_credits(bob, 20).unwrap();
    db.add_credits(bob, 30).unwrap();
    db.add_credits(bob, 40).unwrap();
    db.add_credits(bob, 50).unwrap();

    let bob_info = db.get_info(bob).unwrap();
    assert_eq!(bob_info.userdata.credits, 153);

    assert_eq!(db.total_premium_credits().sum, 0);

    db.add_credits(alice, 10).unwrap();
    assert_eq!(db.total_premium_credits().sum, 10);

    db.reward_premium(1.5).unwrap();
    assert_eq!(db.total_premium_credits().sum, 15);

    // for entry in db.get_snapshot().it {
    //     println!("{:5} {:9} has {:04} credits", entry.name, if *entry.premium { "(premium)" } else { ""}, entry.credits);
    // }
}
