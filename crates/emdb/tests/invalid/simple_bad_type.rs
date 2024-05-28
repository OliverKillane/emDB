use emdb::macros::emql;


emql! {
    table people {
        name: String,
        age: u8,
        preference: super::RGB,
        score: i32,
    } @ [unique(name) as unique_names, pred(age < 100 && age > 10) as reasonable_ages]

    query cool_bob(no_table: ref countries) {
        use people
            |> take(1)
            |> collect(p as type person)
            ~> return;
    }
}

fn main() {}