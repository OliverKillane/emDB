//! Used to check the generated code type checks on mutable values that are not [Copy]
//! - If code generation borks a reference out, it may silently compile because the value 
//!   was copy, hence as String is not, we use that to test here.
use emdb::macros::emql;

emql! {
    impl my_db as Serialized;

    table strings {
        imm_string: String,
        mut_string: String,
    }

    query add_string(s: &'static str) {
        row(
            imm_string: String = String::from(s), 
            mut_string: String = String::from(s), 
        ) ~>
            insert(strings as ref string_id) ~> return;
    }

    query mutate_string(id: ref strings, new: &'static str) {
        row(
            id: ref strings = id,
        ) 
            ~> update(id use mut_string = String::from(new)) 
            ~> deref(id as string_data use mut_string)
            ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    let id = db.add_string("hello").string_id;
    db.mutate_string(id, "cool").expect("correct reference").string_data;
}