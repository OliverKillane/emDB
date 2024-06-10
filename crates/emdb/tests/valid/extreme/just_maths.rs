use emdb::macros::emql;

emql! {
    impl my_db as Serialized;

    query just_maths(x: i32) {
        row(x: i32 = x + 1)
            ~> map(y: u64 = (x * x) as u64)
            ~> map(z: bool = y > 1600)
            ~> return;
    } 
}


pub fn test() {
    let mut ds = my_db::Datastore::new();
    let db = ds.db();

    assert!(!db.just_maths(39).z);
}
