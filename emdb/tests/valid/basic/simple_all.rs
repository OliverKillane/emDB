use emdb::emql;

emql! {
    impl planviz as my_db_graph;

    table simple {
        a: i32,
        b: String,
        c: (u32, i32),
    } @ [unique(a), pred(c.0 > c.1) as c_predicate, pred(b.len() < 10) as b_length]

    // cool comment here
    query insert(a_initial: i32) {
        row(a: i32 = a_initial, b: String = "hello".to_string(), c: (u32, i32) = (0, 0))
            ~> insert(simple as ref it)
            ~> return;
    }

    table other {} @ [pred(1 + 1 == 2)]

    query update_b(new_b: String) {
        ref simple as simple_ref
            |> update(simple_ref use b = new_b) 
            |> collect(it as type foo)
            ~> return;
    }

    query single_maths() {
        row(a: i32 = 0, b: i32 = 2) 
            ~> map(c: i32 = a + b) 
            ~> let x;

        use x 
            ~> map(z: i32 = c*c) 
            ~> return;
    }
}

fn main() {}