use crate::{
    columns::{AssocBlocksCopy, PrimaryRetainCopy, PrimaryThunderDomeTrans, PrimaryThunderdome},
    groups::{Group, GroupConfig},
    table::Table,
};

use super::*;

/// Generates a table data structure but does not make use of mutability.
/// - Is otherwise identical to the [super::MutabilitySelector].
pub struct CopySelector;

impl SelectorImpl for CopySelector {
    fn select_table(
        &self,
        SelectOperations {
            name,
            transactions,
            deletions,
            fields,
            uniques,
            predicates,
            updates,
            gets,
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
                PrimaryRetainCopy { block_size: 1024 }.into()
            }
        } else {
            AssocBlocksCopy { block_size: 1024 }.into()
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
