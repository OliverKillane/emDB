use emdb::macros::emql;


emql! {
    impl my_db as Serialized;

    table values {
        big_number: u8,
        unused_string: String,
    }

    query add_string(unused_string: String, big_number: u8,) {
        row(
            unused_string: String = unused_string,
            big_number: u8 = big_number,
        ) ~> insert(values as ref value_id)
        ~> return;
    }

    query sum_numbers() {
        ref values as values_ref
            |> deref(values_ref as cool use big_number)
            |> map(number: usize = *(cool.big_number) as usize)
            |> combine(use left + right in number[0] = [left.number + right.number])
            ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    const INSERTS: usize = 101;
    const INSERT_NUM: u8 = 25;
    for i in 0..INSERTS {
        let _: my_db::tables::values::Key = db.add_string(String::from("bob"), INSERT_NUM).value_id;
    }

    let count = db.sum_numbers().unwrap().number;
    assert_eq!(count, INSERTS * INSERT_NUM as usize);
}