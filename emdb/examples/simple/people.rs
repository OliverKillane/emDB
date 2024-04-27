#![allow(unused_variables)]
use emdb::emql;

#[allow(dead_code)]
enum RGB {
    Red,
    Blue,
    Green,
}

emql! {
    impl my_db as SemCheck;

    table people {
        name: String,
        age: u8,
        fav: super::super::RGB,
        score: i32,
    } @ [
        unique(name) as unique_names, 
        pred(age < 100 && age > 10) as reasonable_ages
    ]

    query add_new_person(name: String, age: u8, fav: super::super::RGB) {
        row(
            name: String = name, 
            age: u8 = age, 
            fav: super::super::RGB = fav, 
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
}

fn main() {}