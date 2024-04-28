<img src="./docs/logo/simple.drawio.svg" alt="emDB" title="emdb logo" width="300"/>

## What is this?
The `emdb` library to used the emdb project.

## How to use <img src="./docs/logo/simple.drawio.svg" alt="emDB" style="vertical-align: middle;" title="emdb logo" width="70"/>
1. Add the dependency to your `Cargo.toml`
    ```toml
    [dependencies]
    emdb = ...
    ```
2. Use the `emQL` macro to describe your schema and queries. Normal rust expressions and types can be embedded, and errors are propagated to `rustc` and your IDE.
    ```rust
    use emdb::emql;

    enum RGB { Red, Blue, Green }

    emql! {
        impl People as Simple;

        table people {
            name: String,
            age: u8,
            fav: super::RGB,
            score: i32,
        } @ [
            unique(name) as unique_names, 
            pred(age < 100 && age > 10) as reasonable_ages
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
                ~> update(p use score = score + diff)
                ~> map(score: i32 = person.score)
                ~> return;
        }
    }
    ```
3. Use your database in normal rust.
    ```rust
    fn foo() {
        let mut db = People::DB::new();

        let bob_ref = db.add_new_person(
            String::from("bob"), 24, RGB::Red
        ).unwrap();
        
        db.year_passes().unwrap(); 

        let bob_old_score = db.update_scores(bob_ref, 23).unwrap();
    }
    ```

*See more in [examples](./examples/)*
