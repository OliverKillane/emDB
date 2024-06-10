use emdb::macros::emql;

emql! {
    impl my_db as Serialized;

    table red {
        value: usize,
    }

    query add_red(data: usize) {
        row(value: usize = data)
            ~> insert(red as ref new_key);
    }

    query data_counts() {
        ref red as blagh 
            |> count(bob)
            ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    const INSERTS: usize = 101;

    for i in 0..INSERTS {
        let _: () = db.add_red(i);
    }

    let count = db.data_counts().bob;
    assert_eq!(count, INSERTS);
}