use crate::{
    columns::{AssocBlocks, PrimaryThunderDomeTrans, PrimaryThunderdome},
    groups::{Group, GroupConfig, MutImmut},
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
        let primary_fields = MutImmut {
            imm_fields: Vec::new(),
            mut_fields: utils::convert_fields(fields),
        };

        let prim_col = if deletions {
            if transactions {
                PrimaryThunderDomeTrans.into()
            } else {
                PrimaryThunderdome.into()
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
