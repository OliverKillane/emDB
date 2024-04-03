use emdb::emql;

emql! {
    impl planviz as mydb;

    table simple {
        a: i32,
        b: String,
        c: (u32, i32),
    }
}

fn main() {}