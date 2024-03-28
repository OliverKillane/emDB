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
}