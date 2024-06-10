use std::collections::{HashMap, HashSet};

use emdb::macros::emql;

emql! {
    impl my_db as Serialized;

    table customers {
        forename: String,
        surname: String,
        age: u8,
    } @ [pred(*age < 255) as sensible_ages]

    query customer_age_brackets() {
        use customers
            |> groupby(age for let people in {
                use people
                    |> collect(people as type age_group)
                    ~> map(age_bracket: u8 = *age, group: type age_group = people)
                    ~> return;
            })
            |> filter(*age_bracket > 16)
            |> collect(brackets)
            ~> return;
    }

    query new_customer(forename: &str, surname: &str, age: u8) {
        row(
            forename: String = String::from(forename),
            surname: String = String::from(surname),
            age: u8 = age
        )
            ~> insert(customers as ref name)
            ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    db.new_customer("Alice", "Smith", 25).unwrap();
    db.new_customer("Bob", "Jones", 25).unwrap();
    db.new_customer("Charlie", "Brown", 40).unwrap();
    db.new_customer("David", "White", 50).unwrap();
    db.new_customer("Eve", "Black", 50).unwrap();

    let brackets: HashMap<u8, HashSet<(&String, &String)>> = db.customer_age_brackets().brackets.into_iter().map(
        |data| (data.age_bracket, data.group.into_iter().map(|v| (v.forename, v.surname)).collect())
    ).collect();

    assert!(brackets[&25].contains(&(&String::from("Alice"), &String::from("Smith"))));
    assert!(brackets[&25].contains(&(&String::from("Bob"), &String::from("Jones"))));
    assert!(brackets[&40].contains(&(&String::from("Charlie"), &String::from("Brown"))));
    assert!(brackets[&50].contains(&(&String::from("David"), &String::from("White"))));
    assert!(brackets[&50].contains(&(&String::from("Eve"), &String::from("Black"))));
}
