#![allow(unused_variables)]
use emdb::macros::emql;

emql! {
    impl my_db as SemCheck;

    table simple {
        a: i32,
        b: String,
        c: (u32, i32),
    }
}

fn main() {}
