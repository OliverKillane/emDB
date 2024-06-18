use emdb::macros::emql;

const CURRENT_YEAR: u16 = 2024; // TODO: update in 2025 and stop mocking what works

emql!{
    impl my_db as Serialized { 
        // options can be specified for implementations
        aggressive_inlining = on,
    };

    table people {
        name: String,
        birth_year: u16,
        friend: Option<String>,
    } @ [
        // constraints can be applied for unique columns, table length limit and row predicates. 
        // provided names are used in the error return types on constraint breach
        unique(name) as unique_names, 
        limit(1000) as max_people, 
        pred(name.len() < 100) as sensible_name_length 
    ]

    // Each query is expressed as a method on the generated database object
    //  - The self borrow is determined by the mutation operations in the query
    //  - The return type is inferred from the query, and is free for the backend 
    //    to make optimisation decisions on.

    // DESCRIPTION: Get the number of friendships between people born in the same decade
    query same_year_friendships() {
        use people 
            |> map(
                name: &'db String = name, 
                birth_decade: u16 = (birth_year / 10) * 10,
                friend: &'db Option<String> = friend
            )
            |> groupby(birth_decade for let same_year_people in {
                // Queries are not nested like SQL, but are a DAG, streams can be duplicated.
                use same_year_people |> fork(let friend_side, friended_side);

                join(use friend_side [
                    // predicate (inner), cross and equi (inner) joins are supported
                    inner pred { 
                        if let Some(friend_name) = &left.friend {
                            friend_name == right.name
                        } else {
                            false
                        }
                    }
                ] use friended_side)
                    |> count(num_friendships)
                    ~> map(decade: u16 = birth_decade, num_friendships: usize = num_friendships)
                    ~> return;
                }
            )
            |> collect(decades)
            ~> return; // The return type is inferred and includes the errors that can occur.
    }

    // DESCRIPTION: update an incorrect birth year using a name, and return a row reference
    query fix_birth_year(name: &'qy String, new_year: u16) {
        row(
            // using a reference type for input ('qy is the lifetime of the borrow of the database)
            name: &'qy String = name,
        )
            ~> unique(name for people.name as ref person) // access to columns with unique indexes
            ~> update(person use birth_year = new_year)
            ~> map(person: ref people = person) // use of a row reference type
            ~> return;
    }

    // DESCRIPTION: Add a new person, and return the number of people who declare them a friend. 
    query new_friendship(user_name: &str, friend: Option<String>) {
        row(
            name: String = String::from(user_name),
            friend: Option<String> = friend,

            // use of a user defined constant.
            birth_year: u16 = crate::valid::complex::thesis_example::CURRENT_YEAR,
        )
            ~> insert(people as ref new_person_id)
            ~> let person_id; // streams and singles can be assigned to (single read) variables.
        
        // can use the set variable once
        use person_id
            ~> lift(
                // the lift operator opens a new context, with the stream/single's record fields 
                // available to use in expressions, here it means we can refer to `new_person_id` 
                // in an expression, while also using `user_name` from the query's top context
                ref people as other_person_id
                       // access with a row reference to the column `friend`
                    |> deref(other_person_id as other_person use friend)
                    |> filter(
                        if let Some(other_name) = &other_person.friend { 
                            other_name == user_name && *other_person_id != new_person_id 
                        } else {
                            false 
                        })
                    |> count(current_frienders)
                    ~> return;
            ) ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    assert_eq!(db.new_friendship("Alice", Some("Bob".to_string())).unwrap().current_frienders, 0);
    assert_eq!(db.new_friendship("Bob", Some("Alice".to_string())).unwrap().current_frienders, 1);

    // does not count self as a friend
    assert_eq!(db.new_friendship("Jim", Some("Jim".to_string())).unwrap().current_frienders, 0);

    db.fix_birth_year(&String::from("Jim"), 1990).unwrap();

    let friendships: Vec<(u16, usize)> = db.same_year_friendships().decades.into_iter().map(|v| (v.decade, v.num_friendships)).collect();

    // Jim is friends with jim in the 1990s
    // Alice with Bob, and Bob with alice in the 2000s
    for (decade, num) in friendships {
        println!("{decade}s: {num} friendships");
    }
}