use emdb::database;

database!{
    impl graph as my_db;

    table simple {
        a: i32,
        b: String,
        c: (u32, i32),
    } @ [unique(a), pred(c.0 > c.1) as c_predicate, pred(b.len() < 10) as b_length]

    query cool() {
        use simple |> collect ~> return;
    }

    // wrong_type goes to compiler bug for ~
    query badly_written(wrong_type: i32, other_type: i32) {
        use nonexistent_table 
            |> collect 
            ~> return;

        ref simple 
            |> deref(simple as cool) 
            |> map(val1: i32 = cool.a, myref: ref simple = simple)
            |> collect 
            ~> return;
    }
}