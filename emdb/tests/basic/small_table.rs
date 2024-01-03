use emdb::database;

database! {
    impl coolbackend as mydb;

    table simple {
        a: i32,
        b: String,
        c: (u32, i32),
    }
}
