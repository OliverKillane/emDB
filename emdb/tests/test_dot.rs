use emdb::emql;

#[allow(dead_code)]
enum RGB {
    Red,
    Blue,
    Green,
}

emql! {
    impl my_db as Planviz{path = "cool.dot", display_types = on, display_ctx_ops = on, display_control = on};

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
            |> collect(it as type foo) 
            ~> return;

        use x2 
            |> sort(x desc)
            |> fork(let z1, z2);

        use z1 |> sort(x desc) |> sort(x asc) |> let z3;

        union(use z3, z2) |> collect(x as type bing);
    }

    query complex2() {
        use cool |> map(x: i32 = name.len()) |> let x;
        use cool |> map(x: i32 = something) |> let y;
        union(use x, y) |> collect(id as type foo) ~> return;
    }
}

fn main() {}