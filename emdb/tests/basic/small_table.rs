use emdb::database;

database! {
    impl graph as mydb;

    table simple {
        a: i32,
        b: String,
        c: (u32, i32),
    }
}
