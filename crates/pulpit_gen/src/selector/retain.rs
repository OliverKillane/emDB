use super::*;

use crate::{
    columns::PrimaryThunderDomeTrans,
    groups::{Group, GroupConfig},
    table::Table,
};

/// Generates a table data structure using the provided updates to determine
/// field mutability, and considering use of deletions and transactions.
/// - Assumes the cost of accumulating unused immutable fields (from
///   [`crate::columns::PrimaryRetain`]) is negated by the cost of referencing on `get`
pub fn selector(
    SelectOperations {
        name,
        transactions,
        deletions,
        fields,
        uniques,
        predicates,
        updates,
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
        name,
        transactions,
        deletions,
        public,
    }
}
