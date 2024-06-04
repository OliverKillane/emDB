//! For manually debugging generated code.
//!
//! - Ensure that the proc macro is built. In vscode on the bottom bar you can
//!   hover over `rust-analyzer` and click `Rebuild Proc Macros`
//! - Saving this file should re-run the emql macro, to generate outputs.
#![allow(unreachable_code)]
use emdb::macros::emql;

emql! {
    impl my_db as SimpleSerialized{debug_file = "emdb/tests/code.rs"};

}

fn main() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();
}
