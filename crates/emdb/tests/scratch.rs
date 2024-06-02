//! For manually debugging generated code.
//!
//! - Ensure that the proc macro is built. In vscode on the bottom bar you can
//!   hover over `rust-analyzer` and click `Rebuild Proc Macros`
//! - Saving this file should re-run the emql macro, to generate outputs.
#![allow(unreachable_code)]
use emdb::macros::emql;

// emql! {
//     impl debug_code as SimpleSerialized{debug_file = "emdb/tests/code.rs"};

//     // Use the vscode dots view to see preview update live on save
//     // impl debug_graph as PlanViz{path = "emdb/tests/debug/graph.dot", display_types = on, display_ctx_ops = on, display_control = on};

//     // write query to check here!
//     table customers {
//         forename: String,
//         surname: String,
//         age: u8,
//         id: usize,
//     } @ [ pred(*age < 120) as sensible_ages, unique(id) as unique_id ]

//     query get_unique_customer(name: &'qy str, id: usize) {
//         row(name: &'qy str = name, ident: usize = id) ~> unique(ident for customers.id as ref cust_ref);
//     }

//     query drop_all() {
//         ref customers as cust |> delete(cust); 
//     }

//     query update_name() {
//         ref customers as cust_key
//             |> deref(cust_key as cust_data)
//             |> update(cust_key use age = cust_data.age + 1);
//     }

//     query insert_name(fname: String, id: usize, age: u8) {
//         row(
//             forename: String = fname,
//             surname: String = String::from("Smith"),
//             age: u8 = age,
//             id: usize = id
//         )
//             ~> insert(customers as ref cust_ref)
//             ;
//     }
// }
