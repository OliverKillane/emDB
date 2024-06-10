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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
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
        // debug_file = "emdb/tests/code.rs",
        // op_impl = Parallel,
        // table_select = Thunderdome,
    };

    table customers {
        forename: String,
        surname: String,
        age: u8,
    } @ [pred(*age < 255) as sensible_ages]

    query customer_age_brackets() {
        use customers
            |> groupby(age for let people in {
                use people
                    |> collect(people as type age_group)
                    ~> map(age_bracket: u8 = *age, group: type age_group = people)
                    ~> return;
            })
            |> filter(*age_bracket > 16)
            |> collect(brackets)
            ~> return;
    }

    query new_customer(forename: &str, surname: &str, age: u8) {
        row(
            forename: String = String::from(forename),
            surname: String = String::from(surname),
            age: u8 = age
        )
            ~> insert(customers as ref name)
            ~> return;
    }
}

fn main() {
    // use my_interface::Datastore;
    let mut ds = my_db::Datastore::new();
    let db = ds.db();
}
