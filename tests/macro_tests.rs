use emdb::database;

database! {
    impl coolbackend as mydb;

    table cool {
        a: i32,
    } @ [unique(id)]

    query bob(a: i32) {
        ref table |> assert(it < 3) |> delete() |> return |> delete();
        // ref bob |> map(t: i32 = it.val) |>
        // unique(tableb )
    }
}
