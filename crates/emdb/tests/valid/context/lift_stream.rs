use emdb::macros::emql;

emql! {
    impl my_db as Serialized;

    table customers {
        forename: String,
        surname: String,
        age: u8,
        bonus_points: i32,
    } @ [ pred(*age < 255) as sensible_ages ]

    table family_bonus {
        surname: String,
        bonus: i32
    } @ [ unique(surname) as unique_surnames_cons ]

    query customer_age_brackets() {
        ref customers as ref_cust
            |> deref(ref_cust as person)
            |> update(ref_cust use bonus_points = person.bonus_points + 1)
            |> lift(
                row(surname: String = person.surname.clone())
                    ~> unique(surname for family_bonus.surname as ref family_ref)
                    ~> deref(family_ref as family)
                    ~> update(family_ref use bonus = family.bonus + 1);

                row() ~> return; // void return
            );
    }

    query add_customer(forename: String, surname: String, age: u8) {
        row(
            forename: String = forename,
            surname: String = surname,
            age: u8 = age,
            bonus_points: i32 = 0
        )
            ~> insert(customers as ref name)
            ~> return;
    }

    query add_family(surname: String) {
        row(surname: String = surname, bonus: i32 = 0)
            ~> insert(family_bonus as ref name)
            ~> return;
    }

    query get_family(family: ref family_bonus) {
        row(family: ref family_bonus = family)
            ~> deref(family as family_val)
            ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    db.add_customer("Alice".to_string(), "Smith".to_string(), 25).unwrap();
    db.add_customer("Bob".to_string(), "Smith".to_string(), 30).unwrap();
    db.add_customer("Charlie".to_string(), "Smith".to_string(), 35).unwrap();
    let smiths = db.add_family("Smith".to_string()).unwrap().name;
    
    db.add_customer("David".to_string(), "Jones".to_string(), 40).unwrap();
    let joneses = db.add_family("Jones".to_string()).unwrap().name;

    db.customer_age_brackets().unwrap();

    assert_eq!(db.get_family(smiths).unwrap().family_val.bonus, 3);
    assert_eq!(db.get_family(joneses).unwrap().family_val.bonus, 1);
    // Evidently Keeping Up with the Joneses!
}
