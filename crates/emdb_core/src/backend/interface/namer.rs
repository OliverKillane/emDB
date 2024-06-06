use syn::Ident;

use crate::utils::misc::new_id;


pub struct InterfaceNamer {
    pub trait_database: Ident,
    pub trait_datastore: Ident,
    pub trait_datastore_method_new: Ident,
    pub trait_datastore_method_db: Ident,
    pub trait_any: Ident,
}

impl InterfaceNamer {
    pub fn new() -> Self {
        Self {
            trait_database: new_id("Database"),
            trait_datastore: new_id("Datastore"),
            trait_datastore_method_new: new_id("new"),
            trait_datastore_method_db: new_id("db"),
            trait_any: new_id("Any")
        }
    }
}