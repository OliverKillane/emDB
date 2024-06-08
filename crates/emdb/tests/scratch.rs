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
    impl my_interface as Interface{
        traits_with_db = { },
    };
    impl my_db as Serialized{
        debug_file = "emdb/tests/debug/code.rs",
        // interface = my_interface,
        // pub = on,
        ds_name = EmDBDebug,
    };
    impl code_display as PlanViz{
        path = "emdb/tests/debug/code.dot",
        types = off,
        ctx = on,
        control = on,
    };

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
            |> lift(
                row(surname: String = person.surname.clone())
                    ~> unique(surname for family_bonus.surname as ref family_ref)
                    ~> deref(family_ref as family)
                    ~> update(family_ref use bonus = family.bonus + 1);

                row() ~> return; // void return
            );
    }

    query add_customer(forename: String, surname: String, age: u8) {
        row(
            forename: String = forename,
            surname: String = surname,
            age: u8 = age,
            bonus_points: i32 = 0
        )
            ~> insert(customers as ref name)
            ~> return;
    }

    query add_family(surname: String) {
        row(surname: String = surname, bonus: i32 = 0)
            ~> insert(family_bonus as ref name)
            ~> return;
    }

    query get_family(family: ref family_bonus) {
        row(family: ref family_bonus = family)
            ~> deref(family as family_val)
            ~> return;
    }
}

fn main() {
    let mut ds = my_db::EmDBDebug::new();
    let db = ds.db();
}
