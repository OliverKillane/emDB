use copy_string::Database;
use emdb::macros::emql;

emql! {
    impl copy_string as Interface{
        pub = on,
    };
    impl emdb_ref_impl as Serialized{
        interface = copy_string,
        pub = on,
        ds_name = EmDBRef,
        aggressive_inlining = on,
        op_impl = Iter,
    };
    impl emdb_copy_impl as Serialized{
        interface = copy_string,
        pub = on,
        ds_name = EmDBCopy,
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
        use values
            |> map(unrelated_value: () = ())
            |> count(count)
            ~> return;
    }

    query count_values_deref() {
        ref values as values_ref
            |> deref(values_ref as bob use )
            |> map(unrelated_value: () = ())
            |> count(count)
            ~> return;
    }

    query count_refs() {
        ref values as values_ref
            |> map(unrelated_value: () = ())
            |> count(count)
            ~> return;
    }
}

pub fn populate_database<DS: copy_string::Datastore>(values: usize, str_len: usize) -> DS {
    let mut ds = DS::new();
    {
        let mut db = ds.db();

        for _ in 0..values {
            let mut s = String::with_capacity(str_len);
            for _ in 0..str_len {
                s.push('a' as char);
            }
            db.add_string(s);
        }
    }
    ds
}
