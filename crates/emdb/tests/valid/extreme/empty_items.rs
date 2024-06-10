use emdb::macros::emql;

emql!{
    impl my_db as Serialized;

    table empty {} @ [ pred(1 + 1 == 2) as always_true_never_checked]

    query empty() {
        // such wow, much empty
    }

    query redundant() {
        row() ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let db = ds.db();

    let () = db.empty();
    let _ = db.redundant(); // is a record with no members
}