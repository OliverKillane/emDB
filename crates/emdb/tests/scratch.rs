#![allow(dead_code, unused_variables)]
//! For manually debugging generated code.
//!
//! - Ensure that the proc macro is built. In vscode on the bottom bar you can
//!   hover over `rust-analyzer` and click `Rebuild Proc Macros`
//! - Saving this file should re-run the emql macro, to generate outputs.
use emdb::macros::emql;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum RGB {
    Red,
    Blue,
    Green,
}

emql! {
    // impl my_interface as Interface{
    //     traits_with_db = { },
    // };
    // impl my_db as Serialized{
    //     // debug_file = "emdb/tests/code.rs",
    //     // interface = my_interface,
    //     // pub = on,
    //     ds_name = EmDBDebug,
    //     // aggressive_inlining = on,
    // };
    // impl code_display as PlanViz{
    //     path = "emdb/tests/debug/code.dot",
    //     types = off,
    //     ctx = on,
    //     control = on,
    // };

    impl my_db as Serialized {
        // debug_file = "emdb/tests/code2.rs",
        // op_impl = Parallel,
        // table_select = Thunderdome,
    };
}

fn main() {
    // use my_interface::Datastore;
    let mut ds = my_db::Datastore::new();
    let db = ds.db();
}
