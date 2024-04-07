use emdb::emql;

emql! {
    table simple {
        a: i32,
        b: String,
        c: (u32, i32),
    }
}

fn main() {}