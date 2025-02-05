#![allow(unused_variables)]
use emdb::macros::emql;
use derefs::{Datastore, Database};

emql! {
    impl derefs as Interface{
        pub = on,
    };
    impl my_db as Serialized{
        interface = derefs,
    };

    table cool {
        name: String,
        something: i32,
    } @ [ unique(name) as individual_names ]

    // returns a reference to the row updated
    query new_cool(name: String) {
        row(name: String = name, something: i32 = 0)
            ~> insert(cool as ref it)
            ~> return;
    }

    query update_cool(id: ref cool) {
        row(id: ref cool = id)
            ~> deref(id as cool_val)
            ~> update(id use something = cool_val.something + 1);
    }

    query get_cool(id: ref cool) {
        row(id: ref cool = id)
            ~> deref(id as cool_val)
            ~> map(score: i32 = cool_val.something)
            ~> return;
    }

    query collect_most_cool() {
        ref cool as cool_ref
            |> deref(cool_ref as cool_vals)
            |> map(sort_on: i32 = cool_vals.something, cool_ref: ref cool = cool_ref)
            |> sort(sort_on desc)
            |> take(10)
            |> map(id: ref cool = cool_ref)
            |> collect(it as type foo)
            ~> map(blah: type foo = it, c: i32 = 0)
            ~> return;
    }

    query complex() {
        use cool
            |> map(x: usize = name.len())
            |> filter(*x > 10)
            |> let larger_than_10;

        use larger_than_10
            |> fork(let x1, x2);

        use x1
            |> take(1)
            |> collect(it)
            ~> return;

        use x2
            |> sort(x desc);
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    for name in &["a", "b", "c"] {
        let _: my_db::tables::cool::Key = db.new_cool(name.to_string()).expect("unique names").it;
    }
    
    let top_10_value = db.collect_most_cool().expect("Correct dereferencing");
    let (c_val, top_cools): (i32, Vec<my_db::tables::cool::Key>) = (top_10_value.c, top_10_value.blah.into_iter().map(|v| v.id).collect());

    let _ = db.get_cool(top_cools[0]).expect("Correct dereferencing").score;
    db.update_cool(top_cools[0]).expect("Correct key");
}
