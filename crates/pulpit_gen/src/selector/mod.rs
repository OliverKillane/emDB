//! For converting a selection of operations into a concrete one for code generation.

// choice of deletion, transaction, updates and mutability.

pub struct SelectOperations {
    pub name: Ident,
    pub transactions: bool,
    pub deletions: bool,
    pub fields: HashMap<Ident, Tokens<Type>>,
    pub uniques: Vec<Unique>,
    pub predicates: Vec<Predicate>,
    pub updates: Vec<Update>,
}

use std::collections::HashMap;

use quote_debug::Tokens;
use syn::{Ident, ItemMod, Type};

use crate::{
    columns::{Append, AppendTrans, AssocBlocks, PrimaryNoPull, PrimaryRetain, Pull, PullTrans},
    groups::{Field, Group, GroupConfig, MutImmut},
    namer::CodeNamer,
    operations::update::Update,
    predicates::Predicate,
    table::Table,
    uniques::Unique,
};

pub enum BackingKind {
    Append(Table<Append>),
    AppendTrans(Table<AppendTrans>),
    Pull(Table<Pull>),
    PullTrans(Table<PullTrans>),
}

pub fn select_basic(
    SelectOperations {
        name,
        transactions,
        deletions,
        mut fields,
        uniques,
        predicates,
        updates,
    }: SelectOperations,
    namer: &CodeNamer,
) -> Tokens<ItemMod> {
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
    } in &updates
    {
        for field in update_fields {
            if let Some(ty) = fields.remove(field) {
                mut_fields.insert(field.clone(), ty);
            }
        }
    }
    let primary_fields = MutImmut {
        imm_fields: convert_fields(fields),
        mut_fields: convert_fields(mut_fields),
    };

    if transactions {
        if deletions {
            Table::<PullTrans> {
                groups: GroupConfig {
                    primary: Group {
                        col: PrimaryRetain { block_size: 1024 }.into(),
                        fields: primary_fields,
                    },
                    assoc: vec![],
                }
                .into(),
                uniques,
                predicates,
                updates,
                name,
            }
            .generate(namer)
        } else {
            Table::<AppendTrans> {
                groups: GroupConfig {
                    primary: Group {
                        col: PrimaryNoPull(AssocBlocks { block_size: 1024 }.into()).into(),
                        fields: primary_fields,
                    },
                    assoc: vec![],
                }
                .into(),
                uniques,
                predicates,
                updates,
                name,
            }
            .generate(namer)
        }
    } else if deletions {
        Table::<Pull> {
            groups: GroupConfig {
                primary: Group {
                    col: PrimaryRetain { block_size: 1024 }.into(),
                    fields: primary_fields,
                },
                assoc: vec![],
            }
            .into(),
            uniques,
            predicates,
            updates,
            name,
        }
        .generate(namer)
    } else {
        Table::<Append> {
            groups: GroupConfig {
                primary: Group {
                    col: PrimaryNoPull(AssocBlocks { block_size: 1024 }.into()).into(),
                    fields: primary_fields,
                },
                assoc: vec![],
            }
            .into(),
            uniques,
            predicates,
            updates,
            name,
        }
        .generate(namer)
    }
}
