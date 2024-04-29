#![allow(unused_variables)]
use emdb::emql;

emql! {
    impl my_db as SemCheck;

    table simple {
        a: i32,
        b: String,
        c: (u32, i32),
    }
}

fn main() {}
