mod my_db {
    #![allow(non_shorthand_field_patterns)]
    use emdb::dependencies::minister::Physical;
    pub mod tables {}
    pub mod queries {}
    pub struct Datastore {}
    impl Datastore {
        pub fn new() -> Self {
            Self {}
        }
        pub fn db(&mut self) -> Database<'_> {
            Database {
                phantom: std::marker::PhantomData,
            }
        }
    }
    pub struct Database<'db> {
        phantom: std::marker::PhantomData<&'db ()>,
    }
}
