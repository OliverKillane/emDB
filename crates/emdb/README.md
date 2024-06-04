<img src="./docs/logo.drawio.svg" alt="emDB" title="emdb logo" width="300"/>

The main `emdb` library to used the emdb project.

## How to use <img src="./docs/logo.drawio.svg" alt="emDB" style="vertical-align: middle;" title="emdb logo" width="70"/>
1. Add the dependency to your `Cargo.toml`
    ```toml
    [dependencies]
    emdb = ...
    ```
2. Use the `emQL` macro to describe your schema and queries. Normal rust expressions and types can be embedded, and errors are propagated to `rustc` and your IDE. And add your own code to interact!
    ```rust
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
        impl my_db as Serialized;

        table people {
            name: String,
            age: u8,
            fav: crate::RGB,
            score: i32,
        } @ [
            unique(name) as unique_names,
            pred(*age < 100 && *age > 10) as reasonable_ages
        ]

        query add_new_person(name: String, age: u8, fav: crate::RGB) {
            row(
                name: String = name,
                age: u8 = age,
                fav: crate::RGB = fav,
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
                |> filter(*p.age > age_cuttoff)
                |> delete(person);
        }
    }

    #[test]
    fn foo() {
        let mut ds = my_db::Datastore::new();
        let mut db = ds.db();

        let bob = db.add_new_person(String::from("Bob"), 23, RGB::Red).expect("empty database").name;
        assert!(db.year_passes().is_ok(), "correct age updating code");
        let jim = db.add_new_person(String::from("Jim"), 99, RGB::Blue).expect("name different Bob").name;
        db.update_scores(bob, 300).expect("Bob is still in the db");

        assert!(db.remove_the_elderly(50).is_ok(), "correct dereferencing emql code");
        assert!(db.update_scores(jim, 3).is_err(), "Mike was removed by the age cuttoff");

        // add a bunch more users
        assert!(db.add_new_person(String::from("Mike"), 34, RGB::Blue).is_ok());
        assert!(db.add_new_person(String::from("Mike"), 47, RGB::Red).is_err(), "added Jim twice");
        assert!(db.add_new_person(String::from("Steven"), 200, RGB::Red).is_err(), "Steven is clearly lying");
        assert!(db.add_new_person(String::from("Alex"), 50, RGB::Green).is_ok());

        for user in db.get_top_scorers(3).expect("Note: the 'use' keyword is a 'ref -> deref' so can error").p.into_iter() {
            println!("{}: {}, {}, {:?}", user.name, user.score, user.age, user.fav);
        }
    }

    fn main() {}
    ```
3. Enjoy the theraputic benefits of type safe, performant code.

*See more in [examples](./examples/)*
