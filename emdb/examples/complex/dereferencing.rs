use emdb::emql;

emql! {
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
            ~> update(id use something = something + 1);
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
            |> map(sort_on: i32 = cool_vals.something)
            |> sort(sort_on desc)
            |> take(10)
            |> map(id: ref cool = cool_ref)
            |> collect(it as type foo)
            ~> map(blah: type foo = it, c: i32 = 0)
            ~> return;
    }

    query complex() {
        use cool
            |> map(x: i32 = cool_val)
            |> filter(x > 10)
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

fn main() {}