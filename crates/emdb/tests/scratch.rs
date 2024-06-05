#![allow(dead_code, unused_variables)]
//! For manually debugging generated code.
//!
//! - Ensure that the proc macro is built. In vscode on the bottom bar you can
//!   hover over `rust-analyzer` and click `Rebuild Proc Macros`
//! - Saving this file should re-run the emql macro, to generate outputs.
use emdb::macros::emql;

#[derive(Debug, Clone, Copy)]
enum RGB {
    Red,
    Blue,
    Green,
}

emql! {
    impl my_db as Serialized{debug_file = "emdb/tests/debug/code.rs"};
    impl code_display as PlanViz{path = "emdb/tests/debug/code.dot", display_types = off, display_ctx_ops = on, display_control = on};

    table data {
        foo: String,
        bing: usize,
        bar: (&'static str, bool),
    }

    query new_data(foo: &str, bing: usize, bar_0: bool) {
        row(
            foo: String = String::from(foo),
            bing: usize = bing,
            bar: (&'static str, bool) = (if bar_0 { "bar" } else { "baz" }, bar_0)
        )
            ~> insert(data as ref new_key)
            ~> return;
    }

    query all_bings() {
        use data
            |> map(bing_val: usize = *bing)
            |> collect(values)
            ~> return;
    }
}

fn main() {
    // let mut ds = my_db::Datastore::new();
    // let mut db = ds.db();
}
