#![allow(unused_variables)]
use emdb::macros::emql;

emql! {
    impl my_db as SimpleSerialized;

    table customers {
        forename: String,
        surname: String,
        age: u8,
    } @ [pred(age < 256) as sensible_ages]

    query customer_age_brackets() {
        use customers
            |> groupby(age for let people in {
                use people
                    |> collect(people as type age_group)
                    ~> map(age_bracket: u8 = age, group: type age_group = people)
                    ~> return;
            })
            |> filter(age_bracket > 16)
            |> collect(brackets)
            ~> return;
    }
}

fn main() {}
