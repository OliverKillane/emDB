//! ## Table Implementation Selection
//! Provides functions for determining the structure of the [`crate::table::Table`] chosen.

use crate::{operations::update::Update, predicates::Predicate, uniques::Unique};
use quote_debug::Tokens;
use std::collections::HashMap;
use syn::{Ident, Type};

pub struct SelectOperations {
    pub name: Ident,
    pub transactions: bool,
    pub deletions: bool,
    pub fields: HashMap<Ident, Tokens<Type>>,
    pub uniques: Vec<Unique>,
    pub predicates: Vec<Predicate>,
    pub updates: Vec<Update>,
    pub public: bool,
}

pub mod basic;
pub mod retain;

mod utils {
    use std::collections::HashMap;

    use quote_debug::Tokens;
    use syn::{Ident, Type};

    use crate::{
        groups::{Field, MutImmut},
        operations::update::Update,
    };

    pub fn determine_mutability(
        updates: &[Update],
        mut fields: HashMap<Ident, Tokens<Type>>,
    ) -> MutImmut<Vec<Field>> {
        fn convert_fields(fields: HashMap<Ident, Tokens<Type>>) -> Vec<Field> {
            fields
                .into_iter()
                .map(|(name, ty)| Field { name, ty })
                .collect()
        }

        let mut mut_fields = HashMap::new();
        for Update {
            fields: update_fields,
            alias: _,
        } in updates
        {
            for field in update_fields {
                if let Some(ty) = fields.remove(field) {
                    mut_fields.insert(field.clone(), ty);
                }
            }
        }
        MutImmut {
            imm_fields: convert_fields(fields),
            mut_fields: convert_fields(mut_fields),
        }
    }
}
