#![allow(unused_variables)]
use emdb::macros::emql;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum RGB {
    Red,
    Blue,
    Green,
}

emql! {
    impl my_db as SimpleSerialized;

    table people {
        name: String,
        age: u8,
        fav: crate::RGB,
        score: i32,
    } @ [
        unique(name) as unique_names,
        pred(*age < 100 && *age > 10) as reasonable_ages
    ]

    query add_new_person(name: String, age: u8, fav: super::RGB) {
        row(
            name: String = name,
            age: u8 = age,
            fav: super::RGB = fav,
            score: i32 = 0
        )
            ~> insert(people as ref name)
            ~> return;
    }

    query year_passes() {
        ref people as p
            |> deref(p as person)
            |> update(p use score = person.score + 1);
    }

    query get_top_scorers(top_n: usize) {
        use people
            |> sort(score asc)
            |> take(top_n)
            |> collect(p as type person)
            ~> return;
    }

    query update_scores(person: ref people, diff: i32) {
        row(p: ref people = person)
            ~> deref(p as person)
            ~> update(p use score = person.score + diff)
            ~> map(score: i32 = person.score)
            ~> return;
    }

    query remove_the_elderly(age_cuttoff: u8) {
        ref people as person
            |> deref(person as p)
            |> filter(*p.age < age_cuttoff)
            |> delete(person);
    }
}

fn main() {}
