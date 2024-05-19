use std::collections::HashMap;

use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ImplItemFn, ItemEnum, ItemMod, TraitItemFn};

use super::{
    columns::{FieldName, Groups, PrimaryKind}, predicates::Predicate, uniques::Unique
};

struct Update {
    fields: HashSet<Ident>,
    alias: Ident,
}

struct UpdateData {
    update_structs: Tokens<ItemMod>,
    error_enum: Tokens<ItemEnum>,
    update_traitfn: Tokens<TraitItemFn>,
    update_implfn: Tokens<ImplItemFn>,
}

impl Update {
    fn generate<Primary: PrimaryKind, const TRANS: bool>(
        &self,
        groups: &Groups<Primary>,
        uniques: &HashMap<FieldName, Unique>,
        predicates: &[Predicate]
    ) -> UpdateData {
        
        // get the unique errors
        uniques.iter().filter_map(|  | {})
        
        // get the predicates
        
        // get all fields (to pass to predicate)

        if TRANS {

        } else {

        }

        UpdateData {
            update_structs: todo!(),
            error_enum: todo!(),
            update_traitfn: todo!(),
            update_implfn: todo!(),
        }
    }
}

/*
pub mod update_types {
    pub mod update_blagh {
        pub struct Update {
            pub field2: i32,
        }
        pub enum Error {

        }
    }
}

pub trait Update {
    fn update_blagh(&mut self, mut update: update_types::update_blagh::Update) -> Result<(), update_types::update_blagh::Error>;
}

impl <'imm> Update for Window<'imm> {
    fn update_blagh(&mut self, mut update: update_types::update_blagh::Update) -> Result<(), update_types::update_blagh::Error> {
        todo!()
    }
}
*/
