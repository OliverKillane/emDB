//! For manually debugging generated code.
//!
//! - Ensure that the proc macro is built. In vscode on the bottom bar you can
//!   hover over `rust-analyzer` and click `Rebuild Proc Macros`
//! - Saving this file should re-run the emql macro, to generate outputs.
#![allow(unreachable_code)]
use emdb::macros::emql;

emql! {
    impl debug_code as SemCheck{debug_file = "emdb/tests/debug/code.rs"};

    // Use the vscode dots view to see preview update live on save
    impl debug_graph as PlanViz{path = "emdb/tests/debug/graph.dot", display_types = on, display_ctx_ops = on, display_control = on};

    // write query to check here!
    table customers {
        forename: String,
        surname: String,
        age: u8,
    } @ [ pred(age < 256) as sensible_ages ]

    query customer_age_brackets() {
        use customers
            |> groupby(age for let people in {
                use people
                    |> collect(people as type age_group)
                    ~> map(age_bracket: u8 = age, group: type age_group = people)
                    ~> return;
            })
            |> filter(age_bracket > 16)
            |> collect(brackets as type brackets)
            ~> return;
    }
}

fn main() {
    debug_code::customer_age_brackets();
}
