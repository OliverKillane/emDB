use emdb::macros::emql;

emql!{
    table foos {
        key: i32,
        val: i32,
    }

    query shadow_time() {
        use foos
            |> groupby(key for let foos /* cannot shadow table here! */ in {
                row()
                    ~> return;
            });
    }
}

fn main() {}