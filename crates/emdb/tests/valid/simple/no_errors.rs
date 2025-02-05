use emdb::macros::emql;

emql!{
    impl my_db as Serialized;

    table data {
        data_foo: String,
        bing: usize,
        bar: (&'static str, bool),
    }

    query new_data(data_foo: &str, bing: usize, bar_0: bool) {
        row(
            data_foo: String = String::from(data_foo),
            bing: usize = bing,
            bar: (&'static str, bool) = (if bar_0 { "bar" } else { "baz" }, bar_0)
        )
            ~> insert(data as ref new_key)
            ~> return;
    }

    query all_bings() {
        use data
            |> map(bing_val: usize = *bing)
            |> collect(values)
            ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    for _ in 0..100 {
        let _: my_db::tables::data::Key = db.new_data("hello", 50, true).new_key;
    }

    let _ = db.all_bings().values.into_iter().map(|v|v.bing_val).collect::<Vec<usize>>();
}
