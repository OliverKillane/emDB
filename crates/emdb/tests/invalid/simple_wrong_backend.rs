use emdb::macros::emql;

emql! {
    impl nonexistent_backend as repeated_impl;
    impl other_missing_backend as repeated_impl;
}

fn main() {}