use emdb::macros::emql;

emql! {
    impl copy_string as Interface{
        pub = on,
    };
    impl emdb_ref_ignore_impl as Serialized{
        interface = copy_string,
        pub = on,
        ds_name = EmDBRefIgnore,
        aggressive_inlining = on,
        op_impl = Iter,
    };
    impl emdb_copy_ignore_impl as Serialized{
        interface = copy_string,
        pub = on,
        ds_name = EmDBCopyIgnore,
        aggressive_inlining = on,
        op_impl = Iter,
        table_select = Copy,
    };

    table values {
        unused_string: String,
    }

    query add_string(unused_string: String) {
        row(
            unused_string: String = unused_string,
        ) ~> insert(values as ref value_id);
    }

    query count_values() {
        use values as ()
            |> map(unrelated_value: () = ())
            |> count(count)
            ~> return;
    }
}

pub fn test() {
    // let mut ds = emdb_copy_ignore_impl::EmDBCopyIgnore::new();
    // let mut db = ds.db();
    // // db.add_string("hello".to_string());
    // // db.add_string("world".to_string());
    // db.count_values();
    // // assert_eq!(db.count_values().count, 2);
}

