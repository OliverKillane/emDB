//! ## Table Implementation Selection
//! Provides functions for determining the structure of the [`crate::table::Table`] chosen.

use crate::{
    limit::Limit,
    operations::{get::Get, update::Update},
    predicates::Predicate,
    table::Table,
    uniques::Unique,
};
use quote_debug::Tokens;
use std::collections::HashMap;
use syn::{Ident, Type};

pub struct SelectOperations {
    pub name: Ident,
    pub transactions: bool,
    pub deletions: bool,
    pub fields: HashMap<Ident, Tokens<Type>>,
    pub uniques: Vec<Unique>,
    pub gets: Vec<Get>,
    pub predicates: Vec<Predicate>,
    pub updates: Vec<Update>,
    pub limit: Option<Limit>,
    pub public: bool,
}

mod mutability;
pub use mutability::MutabilitySelector;
mod thunderdome;
pub use thunderdome::ThunderdomeSelector;
mod copy;
pub use copy::*;

#[enumtrait::store(selector_impl_trait)]
pub trait SelectorImpl {
    fn select_table(&self, ops: SelectOperations) -> Table;
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(table_selector_enum)]
pub enum TableSelectors {
    MutabilitySelector,
    ThunderdomeSelector,

    // For Benchmarks
    CopySelector,
}

#[enumtrait::impl_trait(selector_impl_trait for table_selector_enum)]
impl SelectorImpl for TableSelectors {}

mod utils {
    use std::collections::HashMap;

    use quote_debug::Tokens;
    use syn::{Ident, Type};

    use crate::{
        groups::{Field, MutImmut},
        operations::update::Update,
    };

    pub fn convert_fields(fields: HashMap<Ident, Tokens<Type>>) -> Vec<Field> {
        fields
            .into_iter()
            .map(|(name, ty)| Field { name, ty })
            .collect()
    }

    pub fn determine_mutability(
        updates: &[Update],
        mut fields: HashMap<Ident, Tokens<Type>>,
    ) -> MutImmut<Vec<Field>> {
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
