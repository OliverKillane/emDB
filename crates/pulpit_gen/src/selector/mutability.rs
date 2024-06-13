use crate::{
    columns::{AssocBlocks, PrimaryRetain, PrimaryThunderDomeTrans, PrimaryThunderdome},
    groups::{Group, GroupConfig},
    table::Table,
};

use super::*;

/// Generates a table data structure using the provided updates to determine
/// field mutability, and considering use of deletions and transactions.
/// - Assumes the cost of accumulating unused immutable fields (from
///   [`PrimaryRetain`]) is negated by the cost of referencing on `get`
pub struct MutabilitySelector;

impl SelectorImpl for MutabilitySelector {
    fn select_table(
        &self,
        SelectOperations {
            name,
            transactions,
            deletions,
            fields,
            uniques,
            gets,
            predicates,
            updates,
            public,
            limit,
        }: SelectOperations,
    ) -> Table {
        let primary_fields = utils::determine_mutability(&updates, fields);

        let prim_col = if deletions {
            if primary_fields.imm_fields.is_empty() {
                if transactions {
                    PrimaryThunderDomeTrans.into()
                } else {
                    PrimaryThunderdome.into()
                }
            } else {
                PrimaryRetain { block_size: 1024 }.into()
            }
        } else {
            AssocBlocks { block_size: 1024 }.into()
        };

        Table {
            groups: GroupConfig {
                primary: Group {
                    col: prim_col,
                    fields: primary_fields,
                },
                assoc: vec![],
            }
            .into(),
            uniques,
            predicates,
            updates,
            gets,
            limit,
            name,
            transactions,
            deletions,
            public,
        }
    }
}
