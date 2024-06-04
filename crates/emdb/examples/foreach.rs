#![allow(unused_variables)]
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
            |> foreach(let customer in {
                row(surname: String = person.surname.clone())
                    ~> unique(surname for family_bonus.surname as ref family_ref)
                    ~> deref(family_ref as family)
                    ~> update(family_ref use bonus = family.bonus + 1);

                row() ~> return; // void return
            });
    }
}

fn main() {}
