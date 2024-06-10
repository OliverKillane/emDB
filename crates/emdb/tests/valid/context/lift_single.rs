use emdb::macros::emql;

emql! {
    impl my_db as Serialized { };

    table staff {
        id_serial: usize,
    } @ [ unique(id_serial) as unique_id_card_serial ]

    query add_staff(
        id: usize,
    ) {
        row(
            id_serial: usize = id,
        )
            ~> insert(staff as ref staff_ref)
            ~> lift(
                // wowee! we can use staff_member in this context
                row(member: ref staff = staff_ref) 
                    ~> deref(member as staff_data)
                    ~> return;
            )
            ~> return;
    }
}

pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    db.add_staff(1).unwrap();
}