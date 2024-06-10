use emdb::macros::emql;

const ADD_VALUE: usize = 3;

const fn cool_const() -> usize {
    23
}

emql! {
    impl my_db as Serialized;

    table coordinates  {
        x: i64,
        y: i64,
        z: i64,
    } @ [ limit(crate::valid::simple::limited_table::cool_const() + crate::valid::simple::limited_table::ADD_VALUE) as max_inserts ]

    query new_datapoint(x: i64,
        y: i64,
        z: i64,) {
        row(x: i64 = x, y: i64 = y, z: i64 = z)
            ~> insert(coordinates as ref id)
            ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    for _ in 0..(cool_const() + ADD_VALUE) {
        let _: my_db::tables::coordinates::Key = db.new_datapoint(1, 2, 3).expect("Not at limit!").id;
    }

    // final one over the limit
    assert!(db.new_datapoint(1, 2, 3).is_err());
}
