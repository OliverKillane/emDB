use super::*;

use crate::{
    columns::PrimaryThunderDomeTrans,
    groups::{Group, GroupConfig},
    table::Table,
};

/// Generates a table structure using thunderdome with transaction support.
/// - Does not take advantage of mutability.
pub struct ThunderdomeSelector;

impl SelectorImpl for ThunderdomeSelector {
    fn select_table(
        &self,
        SelectOperations {
            name,
            transactions,
            deletions: _,
            fields,
            uniques,
            predicates,
            updates,
            gets,
            limit,
            public,
        }: SelectOperations,
    ) -> Table {
        Table {
            groups: GroupConfig {
                primary: Group {
                    col: PrimaryThunderDomeTrans.into(),
                    fields: utils::determine_mutability(&updates, fields),
                },
                assoc: vec![],
            }
            .into(),
            uniques,
            predicates,
            updates,
            gets,
            name,
            limit,
            transactions,
            deletions: true,
            public,
        }
    }
}
