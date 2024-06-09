use emdb::macros::emql;

emql! {
    impl my_db as Serialized;

    table values {
        value: i32,
    }

    query add_data(data: i32) {
        row(value: i32 = data)
            ~> insert(values as ref new_key)
            ~> return;
    }

    query sum_data_combine() {
        use values
            |> map(sum: i32 = *value)
            |> combine(use left + right in sum[0] = [left.sum + right.sum])
            ~> return;
    }

    query sum_data_fold() {
        use values
            |> fold(sum: i32 = 0 -> sum + *value)
            ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    for i in 0..100 {
        let _: my_db::tables::values::Key = db.add_data(i).new_key;
    }

    let sum = db.sum_data_fold().sum;
    assert_eq!(sum, 4950);
}
