use emdb::emql;

#[allow(dead_code)]
enum RGB {
    Red,
    Blue,
    Green,
}

emql! {
    table people {
        name: String,
        age: u8,
        preference: super::RGB,
        score: i32,
    } @ [unique(name) as unique_names, pred(age < 100 && age > 10) as reasonable_ages]


    query add_new_person(name: String, age: u8, preference: super::RGB) {
        row(name: String = name, age: u8 = age, preference: super::RGB = preference, score: i32 = 0)
            ~> insert(people as ref name)
            ~> return;
    }

    query year_passes() {
        ref people as p
            |> update(p use score = score + 1);
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
            ~> update(p use score = score + diff)
            ~> deref(p as person)
            ~> map(score: i32 = person.score)
            ~> return;
    }
}

fn main() {}