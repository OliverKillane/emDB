//! For manually debugging generated code.
//!
//! - Ensure that the proc macro is built. In vscode on the bottom bar you can
//!   hover over `rust-analyzer` and click `Rebuild Proc Macros`
//! - Saving this file should re-run the emql macro, to generate outputs.
#![allow(unreachable_code)]
use emdb::macros::emql;

emql! {
    impl my_db as SimpleSerialized{debug_file = "emdb/tests/code.rs"};
    impl code_display as PlanViz{path = "emdb/tests/debug/code.dot", display_types = off, display_ctx_ops = on, display_control = on};

    table customers {
        forename: String,
        surname: String,
        age: u8,
        bonus_points: i32,
    } @ [ pred(*age < 255) as sensible_ages ]

    table family_bonus {
        surname: String,
        bonus: i32
    } @ [ unique(surname) as unique_surnames_cons ]

    query customer_age_brackets() {
        ref customers as ref_cust
            |> deref(ref_cust as person)
            |> update(ref_cust use bonus_points = person.bonus_points + 1)
            |> foreach(let customer in {
                row(surname: String = person.surname.clone())
                    ~> unique(surname for family_bonus.surname as ref family_ref)
                    ~> deref(family_ref as family)
                    ~> update(family_ref use bonus = family.bonus + 1);

                row() ~> return; // void return
            });
    }
}

fn main() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    let x = db.customer_age_brackets();
}
