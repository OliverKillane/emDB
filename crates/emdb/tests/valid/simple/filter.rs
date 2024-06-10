use emdb::macros::emql;


emql! {
    impl my_db as Serialized;

    table data {
        value: i32,
    } @ [unique(value) as unique_values]

    query filter_values(math: i32) {
        row(other_math: i32 = 7)
            ~> lift (
                use data
                    |> filter(**value > other_math)
                    |> collect(filtered)
                    ~> return;
            );
    }
}