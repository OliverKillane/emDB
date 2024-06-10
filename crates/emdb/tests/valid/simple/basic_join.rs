use emdb::macros::emql;

emql! {
    impl my_db as Serialized;

    table people {
        name: String,
        friend: Option<String>,
    } @ [ unique(name) as unique_names]

    query get_friendships() {
        use people |> fork(let person, friend);

        join(use person [
            inner pred {
                if let Some(friend_name) = &left.friend {
                    friend_name == right.name
                } else {
                    false
                }
            }
        ] use friend)
            |> map(peep: &'db String = person.name, buddy: &'db String = friend.name)
            |> collect(friends as type friendship)
            ~> return;
    }

    query new_friendship(name: &str, friend: Option<String>) {
        row(
            name: String = String::from(name),
            friend: Option<String> = friend
        )
            ~> insert(people as ref person)
            ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    db.new_friendship("Alice", Some("Bob".to_string())).unwrap();
    db.new_friendship("Bob", Some("Charlie".to_string())).unwrap();
    db.new_friendship("Charlie", Some("David".to_string())).unwrap();
    db.new_friendship("David", Some("Eve".to_string())).unwrap();
    db.new_friendship("Eve", None).unwrap();

    let friendships: Vec<(&String, &String)> = db.get_friendships().friends.into_iter().map(|v| (v.peep, v.buddy)).collect();

    for (left, right) in friendships {
        println!("{left:8} 💘 {right}")
    }
}