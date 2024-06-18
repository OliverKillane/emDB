use crate::{
    columns::{AssocAppVec, AssocBlocks, AssocPullBlocks, AssocVec, PrimaryRetain},
    groups::{Group, GroupConfig, MutImmut},
    table::Table,
};

use super::*;

/// Splits the fields across separate columns.
/// - Optimises for append only, and immutable values.
pub struct ColumnarSelector;

impl SelectorImpl for ColumnarSelector {
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
        let MutImmut {
            mut imm_fields,
            mut mut_fields,
        } = utils::determine_mutability(&updates, fields);
        let first_field = imm_fields
            .pop()
            .map(|f| (true, f))
            .or_else(|| mut_fields.pop().map(|f| (false, f)));

        let groups = if let Some((imm, field)) = first_field {
            if deletions {
                let assoc = imm_fields
                    .into_iter()
                    .map(|field| Group {
                        col: AssocPullBlocks { block_size: 1024 }.into(),
                        fields: MutImmut {
                            imm_fields: vec![field],
                            mut_fields: vec![],
                        },
                    })
                    .chain(mut_fields.into_iter().map(|field| Group {
                        col: AssocVec.into(),
                        fields: MutImmut {
                            imm_fields: vec![],
                            mut_fields: vec![field],
                        },
                    }))
                    .collect();
                let primary = Group {
                    col: PrimaryRetain { block_size: 1024 }.into(),
                    fields: if imm {
                        MutImmut {
                            imm_fields: vec![field],
                            mut_fields: vec![],
                        }
                    } else {
                        MutImmut {
                            imm_fields: vec![],
                            mut_fields: vec![field],
                        }
                    },
                };
                GroupConfig { primary, assoc }
            } else {
                let assoc = imm_fields
                    .into_iter()
                    .map(|field| Group {
                        col: AssocBlocks { block_size: 1024 }.into(),
                        fields: MutImmut {
                            imm_fields: vec![field],
                            mut_fields: vec![],
                        },
                    })
                    .chain(mut_fields.into_iter().map(|field| Group {
                        col: AssocAppVec.into(),
                        fields: MutImmut {
                            imm_fields: vec![],
                            mut_fields: vec![field],
                        },
                    }))
                    .collect();
                let primary = if imm {
                    Group {
                        col: AssocBlocks { block_size: 1024 }.into(),
                        fields: MutImmut {
                            imm_fields: vec![field],
                            mut_fields: vec![],
                        },
                    }
                } else {
                    Group {
                        col: AssocAppVec.into(),
                        fields: MutImmut {
                            imm_fields: vec![],
                            mut_fields: vec![field],
                        },
                    }
                };

                GroupConfig { primary, assoc }
            }
        } else {
            GroupConfig {
                primary: Group {
                    col: if deletions {
                        PrimaryRetain { block_size: 4096 }.into()
                    } else {
                        AssocAppVec.into()
                    },
                    fields: MutImmut {
                        imm_fields: vec![],
                        mut_fields: vec![],
                    },
                },
                assoc: vec![],
            }
        }
        .into();
        Table {
            groups,
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
