use crate::{
    columns::{AssocBlocks, PrimaryAppendAdapter, PrimaryRetain, PrimaryThunderDomeTrans},
    groups::{Group, GroupConfig, MutImmut},
    table::Table,
};

use super::*;

/// Generates a table data structure using the provided updates to determine 
/// field mutability, and considering use of deletions and transactions.
/// - Assumes the cost of accumulating unused immutable fields (from 
///   [`PrimaryRetain`]) is negated by the cost of referencing on `get` 
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

    let primary_fields = utils::determine_mutability(&updates, fields);

    // Here we assume a 
    let groups = if deletions {
        if primary_fields.imm_fields.is_empty() {
            GroupConfig {
                primary: Group {
                    col: PrimaryThunderDomeTrans.into(),
                    fields: primary_fields,
                },
                assoc: vec![],
            }
        } else {
            GroupConfig {
                primary: Group {
                    col: PrimaryRetain { block_size: 1024 }.into(),
                    fields: primary_fields,
                },
                assoc: vec![],
            }
        }
    } else {
        GroupConfig {
            primary: Group {
                col: PrimaryAppendAdapter.into(),
                fields: MutImmut {
                    imm_fields: vec![],
                    mut_fields: vec![],
                },
            },
            assoc: vec![Group {
                col: AssocBlocks { block_size: 1024 }.into(),
                fields: primary_fields,
            }],
        }
    }
    .into();

    Table {
        groups,
        uniques,
        predicates,
        updates,
        name,
        transactions,
        deletions,
        public,
    }
}
