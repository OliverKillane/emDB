#![allow(dead_code, unused_variables)]
//! For manually debugging generated code.
//!
//! - Ensure that the proc macro is built. In vscode on the bottom bar you can
//!   hover over `rust-analyzer` and click `Rebuild Proc Macros`
//! - Saving this file should re-run the emql macro, to generate outputs.
use emdb::macros::emql;

emql! {
    impl copy_string as Interface{
        pub = on,
    };
    impl code_display as PlanViz{
        path = "emdb/tests/debug/code.dot",
        types = off,
        ctx = on,
        control = on,
    };
    impl my_db as Serialized{
        debug_file = "emdb/tests/debug/code.rs",
        interface = copy_string,
        pub = on,
        ds_name = EmDBRefIgnore,
        aggressive_inlining = on,
        op_impl = Iter,
    };

    table values {
        unused_string: String,
    }

    // query add_string(unused_string: String) {
    //     row(
    //         unused_string: String = unused_string,
    //     ) ~> insert(values as ref value_id);
    // }

    query count_values() {
        use values as ();
            // // |> map(unrelated_value: () = ())
            // // |> count(count)
            // |> collect(foos)
            // ~> return;
    }
}

fn main() {
    // use my_interface::Datastore;
    // let mut ds = my_db::Datastore::new();
    // let db = ds.db();
}
